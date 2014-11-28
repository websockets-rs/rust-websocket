pub use super::util;
pub use super::handshake;
pub use super::server;
pub use super::client;

pub use self::send::{WebSocketSender, WebSocketFragmentSerializer};
pub use self::receive::{WebSocketReceiver, IncomingMessages};

pub mod mask;
pub mod dataframe;
pub mod send;
pub mod receive;

/// Represents a WebSocket message
#[deriving(Show)]
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