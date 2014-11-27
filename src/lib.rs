#![feature(phase)]
extern crate serialize;
extern crate regex;

pub use self::ws::server::{WebSocketServer, WebSocketAcceptor};
pub use self::ws::client::WebSocketClient;

pub mod headers {
	pub use util::header::{HeaderCollection, Headers};
}

pub mod handshake {
	pub use ws::handshake::request::WebSocketRequest;
	pub use ws::handshake::response::WebSocketResponse;
}

pub mod message {
	pub use ws::message::{WebSocketMessage};
	pub use ws::message::{WebSocketSender, WebSocketFragmentSerializer};
	pub use ws::message::{WebSocketReceiver, IncomingMessages};
}

mod ws;
mod util;