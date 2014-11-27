pub use super::util;
pub use super::handshake;
pub use super::server;
pub use super::client;

pub use self::send::WebSocketSender;
pub use self::receive::WebSocketReceiver;

pub mod mask;
pub mod dataframe;
pub mod send;
pub mod receive;

pub enum WebSocketMessage {
	Text(String),
	Binary(Vec<u8>),
	Close,
	Ping,
	Pong,
}