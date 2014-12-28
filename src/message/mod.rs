//! Provides a way to deal with WebSocket messages
#![unstable]
use dataframe::opcode::WebSocketOpcode;
use dataframe::WebSocketDataFrame;
use common::{WebSocketResult, WebSocketError};
use std::option::Option;
use std::str::from_utf8;
use std::io::IoResult;

/// A trait for WebSocket messages
/// 
/// Allows for conversion between WebSocketDataFrames and messages.
pub trait WebSocketMessaging: Send {
	/// Create a new WebSocket message from an opcode and data
	fn from_data(opcode: WebSocketOpcode, data: Vec<u8>) -> WebSocketResult<Self>;
	/// Convert this message into a WebSocketDataFrame
	fn into_dataframe(self) -> WebSocketResult<WebSocketDataFrame>;
}

/// Represents a WebSocket message.
#[deriving(PartialEq, Clone, Show, Send)]
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

impl WebSocketMessaging for WebSocketMessage {
	/// Create a new WebSocketMessage from the given opcode and data
	fn from_data(opcode: WebSocketOpcode, data: Vec<u8>) -> WebSocketResult<WebSocketMessage> {
		Ok(match opcode {
			WebSocketOpcode::Text => { WebSocketMessage::Text(try!(bytes_to_string(data.as_slice()))) }
			WebSocketOpcode::Binary => { WebSocketMessage::Binary(data) }
			WebSocketOpcode::Close => {
				if data.len() > 0 {				
					let status_code = (data[0] as u16 << 8) | data[1] as u16;
					let reason = try!(bytes_to_string(data.slice_from_or_fail(&2u)));
					let close_data = CloseData::new(status_code, reason);
					WebSocketMessage::Close(Some(close_data))
				}
				else {
					WebSocketMessage::Close(None)
				}
			}
			WebSocketOpcode::Ping => { WebSocketMessage::Ping(data) }
			WebSocketOpcode::Pong => { WebSocketMessage::Pong(data) }
			_ => { return Err(WebSocketError::ProtocolError("Unsupported WebSocket opcode".to_string())); }
		})
	}
	/// Convert this message into a WebSocketDataFrame
	fn into_dataframe(self) ->  WebSocketResult<WebSocketDataFrame> {
		let (opcode, data) = match self {
			WebSocketMessage::Text(payload) => { (WebSocketOpcode::Text, payload.into_bytes()) }
			WebSocketMessage::Binary(payload) => { (WebSocketOpcode::Binary, payload) }
			WebSocketMessage::Close(payload) => {(
					WebSocketOpcode::Close,
					match payload {
						Some(payload) => { try!(payload.into_bytes()) }
						None => { Vec::new() }
					} 
			)}
			WebSocketMessage::Ping(payload) => { (WebSocketOpcode::Ping, payload) }
			WebSocketMessage::Pong(payload) => { (WebSocketOpcode::Pong, payload) } 
		};
		Ok(WebSocketDataFrame::new(true, opcode, data))
	}
}

/// Represents data contained in a Close message
#[deriving(PartialEq, Clone, Show)]
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