#![allow(unstable)]
#![warn(missing_docs)]

//! Rust-WebSocket is a WebSocket (RFC6455) library written in Rust.
//!
extern crate hyper;
extern crate url;
extern crate "rustc-serialize" as serialize;
extern crate sha1;
extern crate openssl;

#[macro_use]
extern crate bitflags;

pub use self::default::*;
pub use self::client::WebSocketClient;
pub use self::server::WebSocketServer;
pub use self::handshake::{WebSocketRequest, WebSocketResponse};

pub mod default;
pub mod result;
pub mod handshake;
pub mod header;
pub mod client;
pub mod server;

pub mod ws;