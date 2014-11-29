#![crate_type = "lib"]
#![crate_name = "websocket"]

#![feature(phase)]
extern crate serialize;
extern crate regex;

pub use self::ws::client::WebSocketClient;

/// Structs for manipulation of HTTP headers
pub mod headers {
	pub use util::header::{HeaderCollection, Headers};
}

/// Structs for WebSocket handshake requests and responses
pub mod handshake {
	pub use ws::handshake::request::WebSocketRequest;
	pub use ws::handshake::response::WebSocketResponse;
}

/// Structs for WebSocket messages and the transmission of messages
pub mod message {
	pub use ws::message::{WebSocketMessage};
	pub use ws::message::{WebSocketSender, WebSocketFragmentSerializer};
	pub use ws::message::{WebSocketReceiver, IncomingMessages};
}

mod ws;
mod util;