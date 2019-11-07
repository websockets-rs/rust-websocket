//! Module containing the default implementation for messages.
use crate::dataframe::Opcode;
use crate::result::{WebSocketError, WebSocketResult};
use crate::ws;
use crate::ws::dataframe::DataFrame as DataFrameTrait;
use crate::ws::util::bytes_to_string;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::borrow::Cow;
use std::io;
use std::io::Write;
use std::str::from_utf8;

const FALSE_RESERVED_BITS: &[bool; 3] = &[false; 3];

/// Valid types of messages (in the default implementation)
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Type {
	/// Message with UTF8 test
	Text = 1,
	/// Message containing binary data
	Binary = 2,
	/// Ping message with data
	Ping = 9,
	/// Pong message with data
	Pong = 10,
	/// Close connection message with optional reason
	Close = 8,
}

/// Represents a WebSocket message.
///
/// This message also has the ability to not own its payload, and stores its entire payload in
/// chunks that get written in order when the message gets sent. This makes the `write_payload`
/// allocate less memory than the `payload` method (which creates a new buffer every time).
///
/// Incidentally this (the default implementation of `Message`) implements the `DataFrame` trait
/// because this message just gets sent as one single `DataFrame`.
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Message<'a> {
	/// Type of WebSocket message
	pub opcode: Type,
	/// Optional status code to send when closing a connection.
	/// (only used if this message is of Type::Close)
	pub cd_status_code: Option<u16>,
	/// Main payload
	pub payload: Cow<'a, [u8]>,
}

impl<'a> Message<'a> {
	fn new(code: Type, status: Option<u16>, payload: Cow<'a, [u8]>) -> Self {
		Message {
			opcode: code,
			cd_status_code: status,
			payload,
		}
	}

	/// Create a new WebSocket message with text data
	pub fn text<S>(data: S) -> Self
	where
		S: Into<Cow<'a, str>>,
	{
		Message::new(
			Type::Text,
			None,
			match data.into() {
				Cow::Owned(msg) => Cow::Owned(msg.into_bytes()),
				Cow::Borrowed(msg) => Cow::Borrowed(msg.as_bytes()),
			},
		)
	}

	/// Create a new WebSocket message with binary data
	pub fn binary<B>(data: B) -> Self
	where
		B: IntoCowBytes<'a>,
	{
		Message::new(Type::Binary, None, data.into())
	}

	/// Create a new WebSocket message that signals the end of a WebSocket
	/// connection, although messages can still be sent after sending this
	pub fn close() -> Self {
		Message::new(Type::Close, None, Cow::Borrowed(&[0 as u8; 0]))
	}

	/// Create a new WebSocket message that signals the end of a WebSocket
	/// connection and provide a text reason and a status code for why.
	/// Messages can still be sent after sending this message.
	pub fn close_because<S>(code: u16, reason: S) -> Self
	where
		S: Into<Cow<'a, str>>,
	{
		Message::new(
			Type::Close,
			Some(code),
			match reason.into() {
				Cow::Owned(msg) => Cow::Owned(msg.into_bytes()),
				Cow::Borrowed(msg) => Cow::Borrowed(msg.as_bytes()),
			},
		)
	}

	/// Create a ping WebSocket message, a pong is usually sent back
	/// after sending this with the same data
	pub fn ping<P>(data: P) -> Self
	where
		P: IntoCowBytes<'a>,
	{
		Message::new(Type::Ping, None, data.into())
	}

	/// Create a pong WebSocket message, usually a response to a
	/// ping message
	pub fn pong<P>(data: P) -> Self
	where
		P: IntoCowBytes<'a>,
	{
		Message::new(Type::Pong, None, data.into())
	}

	// TODO: change this to match conventions
	#[allow(clippy::wrong_self_convention)]
	/// Convert a ping message to a pong, keeping the data.
	/// This will fail if the original message is not a ping.
	pub fn into_pong(&mut self) -> Result<(), ()> {
		if self.opcode == Type::Ping {
			self.opcode = Type::Pong;
			Ok(())
		} else {
			Err(())
		}
	}
}

impl<'a> ws::dataframe::DataFrame for Message<'a> {
	#[inline(always)]
	fn is_last(&self) -> bool {
		true
	}

	#[inline(always)]
	fn opcode(&self) -> u8 {
		self.opcode as u8
	}

	#[inline(always)]
	fn reserved(&self) -> &[bool; 3] {
		FALSE_RESERVED_BITS
	}

	fn size(&self) -> usize {
		self.payload.len() + if self.cd_status_code.is_some() { 2 } else { 0 }
	}

	fn write_payload(&self, socket: &mut dyn Write) -> WebSocketResult<()> {
		if let Some(reason) = self.cd_status_code {
			socket.write_u16::<BigEndian>(reason)?;
		}
		socket.write_all(&*self.payload)?;
		Ok(())
	}

	fn take_payload(self) -> Vec<u8> {
		if let Some(reason) = self.cd_status_code {
			let mut buf = Vec::with_capacity(2 + self.payload.len());
			buf.write_u16::<BigEndian>(reason)
				.expect("failed to write close code in take_payload");
			buf.append(&mut self.payload.into_owned());
			buf
		} else {
			self.payload.into_owned()
		}
	}
}

impl<'a> ws::Message for Message<'a> {
	/// Attempt to form a message from a series of data frames
	fn serialize(&self, writer: &mut dyn Write, masked: bool) -> WebSocketResult<()> {
		self.write_to(writer, masked)
	}

	/// Returns how many bytes this message will take up
	fn message_size(&self, masked: bool) -> usize {
		self.frame_size(masked)
	}

	/// Attempt to form a message from a series of data frames
	fn from_dataframes<D>(frames: Vec<D>) -> WebSocketResult<Self>
	where
		D: DataFrameTrait,
	{
		let opcode = frames
			.first()
			.ok_or(WebSocketError::ProtocolError("No dataframes provided"))
			.map(ws::dataframe::DataFrame::opcode)?;
		let opcode = Opcode::new(opcode);

		let payload_size = frames.iter().map(ws::dataframe::DataFrame::size).sum();

		let mut data = Vec::with_capacity(payload_size);

		for (i, dataframe) in frames.into_iter().enumerate() {
			if i > 0 && dataframe.opcode() != Opcode::Continuation as u8 {
				return Err(WebSocketError::ProtocolError(
					"Unexpected non-continuation data frame",
				));
			}
			if *dataframe.reserved() != [false; 3] {
				return Err(WebSocketError::ProtocolError(
					"Unsupported reserved bits received",
				));
			}
			data.append(&mut dataframe.take_payload());
		}

		if opcode == Some(Opcode::Text) {
			if let Err(e) = from_utf8(data.as_slice()) {
				return Err(e.into());
			}
		}

		let msg = match opcode {
			Some(Opcode::Text) => Message {
				opcode: Type::Text,
				cd_status_code: None,
				payload: Cow::Owned(data),
			},
			Some(Opcode::Binary) => Message::binary(data),
			Some(Opcode::Close) => {
				if !data.is_empty() {
					let status_code = (&data[..]).read_u16::<BigEndian>()?;
					let reason = bytes_to_string(&data[2..])?;
					Message::close_because(status_code, reason)
				} else {
					Message::close()
				}
			}
			Some(Opcode::Ping) => Message::ping(data),
			Some(Opcode::Pong) => Message::pong(data),
			_ => return Err(WebSocketError::ProtocolError("Unsupported opcode received")),
		};
		Ok(msg)
	}
}

/// Represents an owned WebSocket message.
///
/// `OwnedMessage`s are generated when the user receives a message (since the data
/// has to be copied out of the network buffer anyway).
/// If you would like to create a message out of borrowed data to use for sending
/// please use the `Message` struct (which contains a `Cow`).
///
/// Note that `OwnedMessage` and `Message` can be converted into each other.
#[derive(Eq, PartialEq, Clone, Debug)]
pub enum OwnedMessage {
	/// A message containing UTF-8 text data
	Text(String),
	/// A message containing binary data
	Binary(Vec<u8>),
	/// A message which indicates closure of the WebSocket connection.
	/// This message may or may not contain data.
	Close(Option<CloseData>),
	/// A ping message - should be responded to with a pong message.
	/// Usually the pong message will be sent with the same data as the
	/// received ping message.
	Ping(Vec<u8>),
	/// A pong message, sent in response to a Ping message, usually
	/// containing the same data as the received ping message.
	Pong(Vec<u8>),
}

impl OwnedMessage {
	/// Checks if this message is a close message.
	///
	///```rust
	///# use websocket_base::OwnedMessage;
	///assert!(OwnedMessage::Close(None).is_close());
	///```
	pub fn is_close(&self) -> bool {
		match *self {
			OwnedMessage::Close(_) => true,
			_ => false,
		}
	}

	/// Checks if this message is a control message.
	/// Control messages are either `Close`, `Ping`, or `Pong`.
	///
	///```rust
	///# use websocket_base::OwnedMessage;
	///assert!(OwnedMessage::Ping(vec![]).is_control());
	///assert!(OwnedMessage::Pong(vec![]).is_control());
	///assert!(OwnedMessage::Close(None).is_control());
	///```
	pub fn is_control(&self) -> bool {
		match *self {
			OwnedMessage::Close(_) => true,
			OwnedMessage::Ping(_) => true,
			OwnedMessage::Pong(_) => true,
			_ => false,
		}
	}

	/// Checks if this message is a data message.
	/// Data messages are either `Text` or `Binary`.
	///
	///```rust
	///# use websocket_base::OwnedMessage;
	///assert!(OwnedMessage::Text("1337".to_string()).is_data());
	///assert!(OwnedMessage::Binary(vec![]).is_data());
	///```
	pub fn is_data(&self) -> bool {
		!self.is_control()
	}

	/// Checks if this message is a ping message.
	/// `Ping` messages can come at any time and usually generate a `Pong` message
	/// response.
	///
	///```rust
	///# use websocket_base::OwnedMessage;
	///assert!(OwnedMessage::Ping("ping".to_string().into_bytes()).is_ping());
	///```
	pub fn is_ping(&self) -> bool {
		match *self {
			OwnedMessage::Ping(_) => true,
			_ => false,
		}
	}

	/// Checks if this message is a pong message.
	/// `Pong` messages are usually sent only in response to `Ping` messages.
	///
	///```rust
	///# use websocket_base::OwnedMessage;
	///assert!(OwnedMessage::Pong("pong".to_string().into_bytes()).is_pong());
	///```
	pub fn is_pong(&self) -> bool {
		match *self {
			OwnedMessage::Pong(_) => true,
			_ => false,
		}
	}
}

impl ws::Message for OwnedMessage {
	/// Attempt to form a message from a series of data frames
	fn serialize(&self, writer: &mut dyn Write, masked: bool) -> WebSocketResult<()> {
		self.write_to(writer, masked)
	}

	/// Returns how many bytes this message will take up
	fn message_size(&self, masked: bool) -> usize {
		self.frame_size(masked)
	}

	/// Attempt to form a message from a series of data frames
	fn from_dataframes<D>(frames: Vec<D>) -> WebSocketResult<Self>
	where
		D: DataFrameTrait,
	{
		Ok(Message::from_dataframes(frames)?.into())
	}
}

impl ws::dataframe::DataFrame for OwnedMessage {
	#[inline(always)]
	fn is_last(&self) -> bool {
		true
	}

	#[inline(always)]
	fn opcode(&self) -> u8 {
		(match *self {
			OwnedMessage::Text(_) => Type::Text,
			OwnedMessage::Binary(_) => Type::Binary,
			OwnedMessage::Close(_) => Type::Close,
			OwnedMessage::Ping(_) => Type::Ping,
			OwnedMessage::Pong(_) => Type::Pong,
		}) as u8
	}

	#[inline(always)]
	fn reserved(&self) -> &[bool; 3] {
		FALSE_RESERVED_BITS
	}

	fn size(&self) -> usize {
		match *self {
			OwnedMessage::Text(ref txt) => txt.len(),
			OwnedMessage::Binary(ref bin) => bin.len(),
			OwnedMessage::Ping(ref data) => data.len(),
			OwnedMessage::Pong(ref data) => data.len(),
			OwnedMessage::Close(ref data) => match data {
				&Some(ref c) => c.reason.len() + 2,
				&None => 0,
			},
		}
	}

	fn write_payload(&self, socket: &mut dyn Write) -> WebSocketResult<()> {
		match *self {
			OwnedMessage::Text(ref txt) => socket.write_all(txt.as_bytes())?,
			OwnedMessage::Binary(ref bin) => socket.write_all(bin.as_slice())?,
			OwnedMessage::Ping(ref data) => socket.write_all(data.as_slice())?,
			OwnedMessage::Pong(ref data) => socket.write_all(data.as_slice())?,
			OwnedMessage::Close(ref data) => match data {
				&Some(ref c) => {
					socket.write_u16::<BigEndian>(c.status_code)?;
					socket.write_all(c.reason.as_bytes())?
				}
				&None => (),
			},
		};
		Ok(())
	}

	fn take_payload(self) -> Vec<u8> {
		match self {
			OwnedMessage::Text(txt) => txt.into_bytes(),
			OwnedMessage::Binary(bin) => bin,
			OwnedMessage::Ping(data) => data,
			OwnedMessage::Pong(data) => data,
			OwnedMessage::Close(data) => match data {
				Some(c) => {
					let mut buf = Vec::with_capacity(2 + c.reason.len());
					buf.write_u16::<BigEndian>(c.status_code)
						.expect("failed to write close code in take_payload");
					buf.append(&mut c.reason.into_bytes());
					buf
				}
				None => vec![],
			},
		}
	}
}

impl From<String> for OwnedMessage {
	fn from(text: String) -> Self {
		OwnedMessage::Text(text)
	}
}

impl From<Vec<u8>> for OwnedMessage {
	fn from(buf: Vec<u8>) -> Self {
		OwnedMessage::Binary(buf)
	}
}

impl<'m> From<Message<'m>> for OwnedMessage {
	fn from(message: Message<'m>) -> Self {
		match message.opcode {
			Type::Text => {
				let convert = String::from_utf8_lossy(&message.payload).into_owned();
				OwnedMessage::Text(convert)
			}
			Type::Close => match message.cd_status_code {
				Some(code) => OwnedMessage::Close(Some(CloseData {
					status_code: code,
					reason: String::from_utf8_lossy(&message.payload).into_owned(),
				})),
				None => OwnedMessage::Close(None),
			},
			Type::Binary => OwnedMessage::Binary(message.payload.into_owned()),
			Type::Ping => OwnedMessage::Ping(message.payload.into_owned()),
			Type::Pong => OwnedMessage::Pong(message.payload.into_owned()),
		}
	}
}

impl<'m> From<OwnedMessage> for Message<'m> {
	fn from(message: OwnedMessage) -> Self {
		match message {
			OwnedMessage::Text(txt) => Message::text(txt),
			OwnedMessage::Binary(bin) => Message::binary(bin),
			OwnedMessage::Close(because) => match because {
				Some(c) => Message::close_because(c.status_code, c.reason),
				None => Message::close(),
			},
			OwnedMessage::Ping(data) => Message::ping(data),
			OwnedMessage::Pong(data) => Message::pong(data),
		}
	}
}

/// Represents data contained in a Close message
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct CloseData {
	/// The status-code of the CloseData
	pub status_code: u16,
	/// The reason-phrase of the CloseData
	pub reason: String,
}

impl CloseData {
	/// Create a new CloseData object
	pub fn new(status_code: u16, reason: String) -> CloseData {
		CloseData {
			status_code,
			reason,
		}
	}
	/// Convert this into a vector of bytes
	pub fn into_bytes(self) -> io::Result<Vec<u8>> {
		let mut buf = Vec::new();
		buf.write_u16::<BigEndian>(self.status_code)?;
		for i in self.reason.as_bytes().iter() {
			buf.push(*i);
		}
		Ok(buf)
	}
}

/// Trait representing the ability to convert
/// self to a `Cow<'a, [u8]>`
pub trait IntoCowBytes<'a> {
	/// Consume `self` and produce a `Cow<'a, [u8]>`
	fn into(self) -> Cow<'a, [u8]>;
}

impl<'a> IntoCowBytes<'a> for Vec<u8> {
	fn into(self) -> Cow<'a, [u8]> {
		Cow::Owned(self)
	}
}

impl<'a> IntoCowBytes<'a> for &'a [u8] {
	fn into(self) -> Cow<'a, [u8]> {
		Cow::Borrowed(self)
	}
}

impl<'a> IntoCowBytes<'a> for Cow<'a, [u8]> {
	fn into(self) -> Cow<'a, [u8]> {
		self
	}
}
