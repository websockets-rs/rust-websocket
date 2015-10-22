//! Module containing the default implementation for messages.

use std::io;
use std::io::Result as IoResult;
use std::io::Write;
use std::iter::{Take, Repeat, repeat};
use result::{WebSocketResult, WebSocketError};
use dataframe::{DataFrame, Opcode, DataFrameRef};
use byteorder::{WriteBytesExt, BigEndian};
use ws::util::message::message_from_data;
use ws;

use std::borrow::Cow;

/// Represents a WebSocket message.
#[derive(PartialEq, Clone, Debug)]
pub struct Message<'a> {
	opcode: Opcode,
	cd_status_code: Option<u16>,
	payload: Cow<'a, [u8]>,
}

impl<'a> Message<'a> {
	pub fn string<S>(data: S) -> Self
	where S: Into<Cow<'a, str>> {
		Message {
			opcode: Opcode::Text,
			cd_status_code: None,
			payload: match data.into() {
				Cow::Owned(msg) => Cow::Owned(msg.into_bytes()),
				Cow::Borrowed(msg) => Cow::Borrowed(msg.as_bytes()),
			},
		}
	}

	pub fn binary<B>(data: B) -> Self
	where B: Into<Cow<'a, [u8]>> {
		Message {
			opcode: Opcode::Binary,
			cd_status_code: None,
			payload: data.into(),
		}
	}

	pub fn close() -> Self {
		Message {
			opcode: Opcode::Close,
			cd_status_code: None,
			payload: Cow::Borrowed(&[0 as u8; 0]),
		}
	}

	pub fn close_because<S>(code: u16, reason: S) -> Self
	where S: Into<Cow<'a, str>> {
		Message {
			opcode: Opcode::Close,
			cd_status_code: Some(code),
			payload: match reason.into() {
				Cow::Owned(msg) => Cow::Owned(msg.into_bytes()),
				Cow::Borrowed(msg) => Cow::Borrowed(msg.as_bytes()),
			},
		}
	}

	pub fn ping<P>(data: P) -> Self
	where P: Into<Cow<'a, [u8]>> {
		Message {
			opcode: Opcode::Ping,
			cd_status_code: None,
			payload: data.into(),
		}
	}

	pub fn pong<P>(data: P) -> Self
	where P: Into<Cow<'a, [u8]>> {
		Message {
			opcode: Opcode::Pong,
			cd_status_code: None,
			payload: data.into(),
		}
	}
}

impl<'a> ws::dataframe::DataFrame for Message<'a> {
    fn is_last(&self) -> bool {
        true
    }

    fn opcode(&self) -> Opcode {
        self.opcode
    }

    fn reserved<'b>(&'b self) -> &'b [bool; 3] {
		&[false; 3]
    }

    fn write_payload<W>(&self, socket: W) -> IoResult<()>
    where W: Write {
		if let Some(reason) = self.cd_status_code {
			try!(socket.write_u16::<BigEndian>(reason));
		}
		try!(socket.write_all(&*self.payload))
    }
}

impl<'a, 'b> ws::Message<'b, Message<'b>> for Message<'a> {

	type DataFrameIterator = Take<Repeat<Message<'b>>>;

	fn from_dataframes<D>(frames: Vec<D>) -> WebSocketResult<Self>
    where D: ws::dataframe::DataFrame {
        unimplemented!();
    }

	fn dataframes(&'b self) -> Self::DataFrameIterator {
		repeat(self.clone()).take(1)
    }

	// /// Attempt to form a message from a series of data frames
	// fn from_dataframes<D>(frames: Vec<D>) -> WebSocketResult<Self>
    // where D: ws::dataframe::DataFrame {
	// 	let mut iter = frames.iter();
    //
	// 	let first = try!(iter.next().ok_or(WebSocketError::ProtocolError(
	// 		"No dataframes provided".to_string()
	// 	)));
    //
	// 	let mut data = first.data.clone();
    //
	// 	if first.reserved != [false; 3] {
	// 		return Err(WebSocketError::ProtocolError(
	// 			"Unsupported reserved bits received".to_string()
	// 		));
	// 	}
    //
	// 	for dataframe in iter {
	// 		if dataframe.opcode != Opcode::Continuation {
	// 			return Err(WebSocketError::ProtocolError(
	// 				"Unexpected non-continuation data frame".to_string()
	// 			));
	// 		}
	// 		if dataframe.reserved != [false; 3] {
	// 			return Err(WebSocketError::ProtocolError(
	// 				"Unsupported reserved bits received".to_string()
	// 			));
	// 		}
	// 		for i in dataframe.data.iter() {
	// 			data.push(*i);
	// 		}
	// 	}
    //
	// 	message_from_data(first.opcode, data)
	// }
	// /// Turns this message into an iterator over data frames
	// fn into_iter(self) -> Take<Repeat<DataFrame>> {
	// 	// Just return a single data frame representing this message.
	// 	let (opcode, data) = match self {
	// 		Message::Text(payload) => (Opcode::Text, payload.into_bytes()),
	// 		Message::Binary(payload) => (Opcode::Binary, payload),
	// 		Message::Close(payload) => (
	// 				Opcode::Close,
	// 				match payload {
	// 					Some(payload) => { payload.into_bytes().unwrap() }
	// 					None => { Vec::new() }
	// 				}
	// 		),
	// 		Message::Ping(payload) => (Opcode::Ping, payload),
	// 		Message::Pong(payload) => (Opcode::Pong, payload),
	// 	};
	// 	let dataframe = DataFrame::new(true, opcode, data);
	// 	repeat(dataframe).take(1)
	// }
}
