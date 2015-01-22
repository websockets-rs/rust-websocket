#![allow(unstable)]
#![warn(missing_docs)]

//! Rust-WebSocket is a WebSocket (RFC6455) library written in Rust.
//!
extern crate hyper;
extern crate url;
extern crate "rustc-serialize" as serialize;
extern crate "sha1-hasher" as sha1;
extern crate openssl;

#[macro_use]
extern crate bitflags;

pub use self::client::Client;
pub use self::server::Server;
pub use self::dataframe::DataFrame;
pub use self::message::Message;
pub use self::stream::WebSocketStream;
pub use self::ws::Sender;
pub use self::ws::Receiver;

pub mod client;
pub mod server;
pub mod dataframe;
pub mod message;
pub mod result;
pub mod stream;
pub mod header;

pub mod ws;