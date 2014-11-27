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
pub enum WebSocketMessage {
	Text(String),
	Binary(Vec<u8>),
	Close,
	Ping,
	Pong,
}