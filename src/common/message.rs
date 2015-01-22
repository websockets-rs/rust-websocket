//! Module containing the default implementation for messages.

use std::io::IoResult;
use std::iter::{Take, Repeat, repeat};
use std::str::from_utf8;
use result::{WebSocketResult, WebSocketError};
use ws::Message;
use super::WebSocketDataFrame;
use super::WebSocketOpcode;

/// Represents a WebSocket message.
#[derive(PartialEq, Clone, Show)]
pub enum WebSocketMessage {
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

impl Message<WebSocketDataFrame> for WebSocketMessage {
	type DataFrameIterator = Take<Repeat<WebSocketDataFrame>>;
	/// Attempt to form a message from an iterator over data frames.
	///
	/// The iterator must only provide data frames constituting
	/// single message and also must return None once the message is
	/// complete.
	fn from_iter<I>(mut iter: I) -> WebSocketResult<Self>
		where I: Iterator<Item = WebSocketDataFrame> {
		
		let first = try!(
			iter.next().ok_or(
				WebSocketError::ProtocolError("Cannot form message with no data frame".to_string())
			)
		);
		
		let mut data = first.data.clone();
		
		match (first.opcode as u8, first.finished) {
			// Continuation opcode on first frame
			(0, _) => return Err(WebSocketError::ProtocolError(
				"Unexpected continuation data frame opcode".to_string()
			)),
			// Fragmented control frame
			(8...15, false) => return Err(WebSocketError::ProtocolError(
				"Unexpected fragmented control frame".to_string()
			)),
			_ => (),
		}
		
		for dataframe in iter {
			if dataframe.opcode != WebSocketOpcode::Continuation {
				return Err(WebSocketError::ProtocolError("Unexpected non-continuation data frame".to_string()));
			}
			data = data + &dataframe.data[];
		}
		
		Ok(match first.opcode {
			WebSocketOpcode::Text => WebSocketMessage::Text(try!(bytes_to_string(&data[]))),
			WebSocketOpcode::Binary => WebSocketMessage::Binary(data),
			WebSocketOpcode::Close => {
				if data.len() > 0 {				
					let status_code = try!((&data[]).read_be_u16());
					let reason = try!(bytes_to_string(data.slice_from(2)));
					let close_data = CloseData::new(status_code, reason);
					WebSocketMessage::Close(Some(close_data))
				}
				else {
					WebSocketMessage::Close(None)
				}
			}
			WebSocketOpcode::Ping => WebSocketMessage::Ping(data),
			WebSocketOpcode::Pong => WebSocketMessage::Pong(data),
			_ => return Err(WebSocketError::ProtocolError("Unsupported opcode received".to_string())),
		})
	}
	/// Turns this message into an iterator over data frames
	fn into_iter(self) -> Take<Repeat<WebSocketDataFrame>> {
		// Just return a single data frame representing this message.
		let (opcode, data) = match self {
			WebSocketMessage::Text(payload) => (WebSocketOpcode::Text, payload.into_bytes()),
			WebSocketMessage::Binary(payload) => (WebSocketOpcode::Binary, payload),
			WebSocketMessage::Close(payload) => (
					WebSocketOpcode::Close,
					match payload {
						Some(payload) => { payload.into_bytes().unwrap() }
						None => { Vec::new() }
					} 
			),
			WebSocketMessage::Ping(payload) => (WebSocketOpcode::Ping, payload),
			WebSocketMessage::Pong(payload) => (WebSocketOpcode::Pong, payload),
		};
		let dataframe = WebSocketDataFrame::new(true, opcode, data);
		repeat(dataframe).take(1)
	}
}

/// Represents data contained in a Close message
#[derive(PartialEq, Clone, Show)]
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
			status_code: status_code,
			reason: reason,
		}
	}
	/// Convert this into a vector of bytes
	pub fn into_bytes(self) -> IoResult<Vec<u8>> {
		let mut buf = Vec::new();
		try!(buf.write_be_u16(self.status_code));
		Ok(buf + self.reason.into_bytes().as_slice())
	}
}

fn bytes_to_string(data: &[u8]) -> WebSocketResult<String> {
	let utf8 = try!(from_utf8(data));
	Ok(utf8.to_string())
}