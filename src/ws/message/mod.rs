pub use super::util;
pub use super::handshake;
pub use super::client;

pub use self::send::{WebSocketSender, WebSocketFragmentSerializer};
pub use self::receive::{WebSocketReceiver, IncomingMessages};

use std::fmt::{Formatter, Result, Show};

pub mod mask;
pub mod dataframe;
pub mod send;
pub mod receive;

/// Represents a WebSocket message
pub enum WebSocketMessage {
	/// A message containing UTF-8 text data
	Text(String),
	/// A message containing binary data
	Binary(Vec<u8>),
	/// A message which indicates closure of the WebSocket connection
	Close(Vec<u8>),
	/// A ping message - should be responded to with a pong message
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