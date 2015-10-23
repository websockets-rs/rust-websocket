//! Module containing the default implementation for messages.
use std::io::Write;
use std::iter::{Take, Repeat, repeat};
use result::{WebSocketResult, WebSocketError};
use dataframe::{DataFrame, Opcode};
use byteorder::{WriteBytesExt, ReadBytesExt, BigEndian};
use ws::util::bytes_to_string;
use ws;

use std::borrow::Cow;

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
	where S: Into<Cow<'a, str>> {
		Message::new(Type::Text, None, match data.into() {
			Cow::Owned(msg) => Cow::Owned(msg.into_bytes()),
			Cow::Borrowed(msg) => Cow::Borrowed(msg.as_bytes()),
		})
	}

    /// Create a new WebSocket message with binary data
	pub fn binary<B>(data: B) -> Self
	where B: IntoCowBytes<'a> {
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
	where S: Into<Cow<'a, str>> {
		Message::new(Type::Close, Some(code), match reason.into() {
			Cow::Owned(msg) => Cow::Owned(msg.into_bytes()),
			Cow::Borrowed(msg) => Cow::Borrowed(msg.as_bytes()),
		})
	}

    /// Create a ping WebSocket message, a pong is usually sent back
    /// after sending this with the same data
	pub fn ping<P>(data: P) -> Self
	where P: IntoCowBytes<'a> {
		Message::new(Type::Ping, None, data.into())
	}

    /// Create a pong WebSocket message, usually a response to a
    /// ping message
	pub fn pong<P>(data: P) -> Self
	where P: IntoCowBytes<'a> {
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
    fn is_last(&self) -> bool {
        true
    }

    fn opcode(&self) -> u8 {
        self.opcode as u8
    }

    fn reserved<'b>(&'b self) -> &'b [bool; 3] {
		FALSE_RESERVED_BITS
    }

	fn payload<'b>(&'b self) -> &'b [u8] {
		unimplemented!();
	}

	fn size(&self) -> usize {
		self.payload.len() + if self.cd_status_code.is_some() {
			2
		} else {
			0
		}
	}

    fn write_payload<W>(&self, socket: &mut W) -> WebSocketResult<()>
    where W: Write {
		if let Some(reason) = self.cd_status_code {
			try!(socket.write_u16::<BigEndian>(reason));
		}
		try!(socket.write_all(&*self.payload));
		Ok(())
    }
}

impl<'a, 'b> ws::Message<'b, Message<'b>> for Message<'a> {

	type DataFrameIterator = Take<Repeat<Message<'b>>>;

	fn dataframes(&'b self) -> Self::DataFrameIterator {
		repeat(self.clone()).take(1)
    }

	/// Attempt to form a message from a series of data frames
	fn from_dataframes<D>(frames: Vec<D>) -> WebSocketResult<Self>
    where D: ws::dataframe::DataFrame {
		let opcode = try!(frames.first().ok_or(WebSocketError::ProtocolError(
			"No dataframes provided".to_string()
		)).map(|d| d.opcode()));

		let mut data = Vec::new();

		for dataframe in frames.iter() {
			if dataframe.opcode() != Opcode::Continuation as u8 {
				return Err(WebSocketError::ProtocolError(
					"Unexpected non-continuation data frame".to_string()
				));
			}
			if *dataframe.reserved() != [false; 3] {
				return Err(WebSocketError::ProtocolError(
					"Unsupported reserved bits received".to_string()
				));
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
			_ => return Err(WebSocketError::ProtocolError(
				"Unsupported opcode received".to_string()
			)),
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
