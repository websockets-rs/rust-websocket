pub use super::util;
pub use super::handshake;
pub use super::server;
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
	Close,
	/// A ping message - should be responded to with a pong message
	Ping,
	/// A pong message
	Pong,
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
			&WebSocketMessage::Close => {
				write!(f, "Close")
			}
			&WebSocketMessage::Ping => {
				write!(f, "Ping")
			}
			&WebSocketMessage::Pong => {
				write!(f, "Pong")
			}
		}
    }
}