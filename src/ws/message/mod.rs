pub use super::util;
pub use super::handshake;
pub use super::client;

use std::fmt::{Formatter, Result, Show};

pub mod mask;
pub mod dataframe;
pub mod send;
pub mod receive;

/// Represents a WebSocket message. When receiving messages, the resulting
/// messages are always entire messages (never fragments). You are able to, 
/// however, send messages as fragments using a WebSocketFragmentSerializer. 
/// These messages will be sent to the remote endpoint and reassembled into
/// a single message.
pub enum WebSocketMessage {
	/// A message containing UTF-8 text data
	Text(String),
	/// A message containing binary data
	Binary(Vec<u8>),
	/// A message which indicates closure of the WebSocket connection.
	/// This message may or may not contain data
	Close(Vec<u8>),
	/// A ping message - should be responded to with a pong message.
	/// Usually the pong message will be sent with the same data as the
	/// received ping message.
	Ping(Vec<u8>),
	/// A pong message
	Pong(Vec<u8>),
}

impl Show for WebSocketMessage {
    fn fmt(&self, f: &mut Formatter) -> Result {
		match self {
			&WebSocketMessage::Text(ref data) => {
				write!(f, "Text({})", data)
			}
			&WebSocketMessage::Binary(ref data) => {
				write!(f, "Binary({})", data.len())
			}
			&WebSocketMessage::Close(ref data) => {
				write!(f, "Close({})", data.len())
			}
			&WebSocketMessage::Ping(ref data) => {
				write!(f, "Ping({})", data.len())
			}
			&WebSocketMessage::Pong(ref data) => {
				write!(f, "Pong({})", data.len())
			}
		}
    }
}