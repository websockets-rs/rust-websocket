//! Module containing the default implementation for messages.
use std::io::Write;
use std::borrow::Cow;
use std::iter::{Take, Repeat, repeat};
use result::{WebSocketResult, WebSocketError};
use dataframe::Opcode;
use byteorder::{WriteBytesExt, ReadBytesExt, BigEndian};
use ws::util::bytes_to_string;
use ws;

const FALSE_RESERVED_BITS: &'static [bool; 3] = &[false; 3];

/// Valid types of messages (in the default implementation)
#[derive(Debug, PartialEq, Clone, Copy)]
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
/// Incidentally this (the default implementation of Message) implements the DataFrame trait
/// because this message just gets sent as one single DataFrame.
#[derive(PartialEq, Clone, Debug)]
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
			payload: payload,
		}
	}

	/// Create a new WebSocket message with text data
	pub fn text<S>(data: S) -> Self
		where S: Into<Cow<'a, str>>
	{
		Message::new(Type::Text,
		             None,
		             match data.into() {
		                 Cow::Owned(msg) => Cow::Owned(msg.into_bytes()),
		                 Cow::Borrowed(msg) => Cow::Borrowed(msg.as_bytes()),
		             })
	}

	/// Create a new WebSocket message with binary data
	pub fn binary<B>(data: B) -> Self
		where B: IntoCowBytes<'a>
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
		where S: Into<Cow<'a, str>>
	{
		Message::new(Type::Close,
		             Some(code),
		             match reason.into() {
		                 Cow::Owned(msg) => Cow::Owned(msg.into_bytes()),
		                 Cow::Borrowed(msg) => Cow::Borrowed(msg.as_bytes()),
		             })
	}

	/// Create a ping WebSocket message, a pong is usually sent back
	/// after sending this with the same data
	pub fn ping<P>(data: P) -> Self
		where P: IntoCowBytes<'a>
	{
		Message::new(Type::Ping, None, data.into())
	}

	/// Create a pong WebSocket message, usually a response to a
	/// ping message
	pub fn pong<P>(data: P) -> Self
		where P: IntoCowBytes<'a>
	{
		Message::new(Type::Pong, None, data.into())
	}

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

	fn payload(&self) -> Cow<[u8]> {
		let mut buf = Vec::with_capacity(self.size());
		self.write_payload(&mut buf).ok();
		Cow::Owned(buf)
	}

	fn size(&self) -> usize {
		self.payload.len() + if self.cd_status_code.is_some() { 2 } else { 0 }
	}

	fn write_payload<W>(&self, socket: &mut W) -> WebSocketResult<()>
		where W: Write
	{
		if let Some(reason) = self.cd_status_code {
			try!(socket.write_u16::<BigEndian>(reason));
		}
		try!(socket.write_all(&*self.payload));
		Ok(())
	}
}

impl<'a, 'b> ws::Message<'b, &'b Message<'a>> for Message<'a> {
	type DataFrameIterator = Take<Repeat<&'b Message<'a>>>;

	fn dataframes(&'b self) -> Self::DataFrameIterator {
		repeat(self).take(1)
	}

	/// Attempt to form a message from a series of data frames
	fn from_dataframes<D>(frames: Vec<D>) -> WebSocketResult<Self>
		where D: ws::dataframe::DataFrame
	{
		let opcode = try!(frames.first()
		                        .ok_or(WebSocketError::ProtocolError("No dataframes provided"))
		                        .map(|d| d.opcode()));

		let mut data = Vec::new();

		for (i, dataframe) in frames.iter().enumerate() {
			if i > 0 && dataframe.opcode() != Opcode::Continuation as u8 {
				return Err(WebSocketError::ProtocolError("Unexpected non-continuation data frame"));
			}
			if *dataframe.reserved() != [false; 3] {
				return Err(WebSocketError::ProtocolError("Unsupported reserved bits received"));
			}
			data.extend(dataframe.payload().iter().cloned());
		}

		Ok(match Opcode::new(opcode) {
		       Some(Opcode::Text) => Message::text(try!(bytes_to_string(&data[..]))),
		       Some(Opcode::Binary) => Message::binary(data),
		       Some(Opcode::Close) => {
			       if data.len() > 0 {
			           let status_code = try!((&data[..]).read_u16::<BigEndian>());
			           let reason = try!(bytes_to_string(&data[2..]));
			           Message::close_because(status_code, reason)
			          } else {
			           Message::close()
			          }
		       }
		       Some(Opcode::Ping) => Message::ping(data),
		       Some(Opcode::Pong) => Message::pong(data),
		       _ => return Err(WebSocketError::ProtocolError("Unsupported opcode received")),
		   })
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
