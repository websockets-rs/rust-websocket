#![warn(missing_docs)]
#![cfg_attr(all(test, feature = "nightly"), feature(test))]

#![deny(unused_mut)]

//! Rust-WebSocket is a WebSocket (RFC6455) library written in Rust.
//!
//! # Clients
//! WebSocket clients make use of the `Client` object, which features two connection
//! functions: `connect()` and `connect_ssl_context()`. See the `Client` struct
//! documentation for more information. These both return a client-side `Request`
//! object which is sent to the server with the `send()` method. The `Request` can
//! be altered, typically using `Request.headers.set()` to add additional headers
//! or change existing ones before calling `send()`.
//!
//! Calling `send()` on a `Request` will obtain a `Response`, which can be checked
//! with the `validate()` method, which will return `Ok(())` if the response is a
//! valid one. Data frames and messages can then be sent by obtaining a `Client`
//! object with `begin()`.
//!
//! # Servers
//! WebSocket servers act similarly to the `TcpListener`, and listen for connections.
//! See the `Server` struct documentation for more information. The `bind()` and
//! `bind_secure()` functions will bind the server to the given `SocketAddr`.
//! `Server` implements Iterator and can be used to iterate over incoming `Request`
//! items.
//!
//! Requests can be validated using `validate()`, and other parts of the request may
//! be examined (e.g. the Host header and/or the Origin header). A call to `accept()`
//! or `fail()` will generate a `Response` which either accepts the connection, or
//! denies it respectively.
//!
//! A `Response` can then be altered if necessary, and is sent with the 'send()`
//! method, returning a `Client` ready to send and receive data frames or messages.
//!
//! # Extending Rust-WebSocket
//! The `ws` module contains the traits and functions used by Rust-WebSockt at a lower
//! level. Their usage is explained in the module documentation.
extern crate hyper;
extern crate unicase;
extern crate url;
extern crate rustc_serialize as serialize;
extern crate openssl;
extern crate rand;
extern crate byteorder;

#[macro_use]
extern crate bitflags;

#[cfg(all(feature = "nightly", test))]
extern crate test;

pub use self::client::Client;
pub use self::server::Server;
pub use self::dataframe::DataFrame;
pub use self::message::Message;
pub use self::stream::WebSocketStream;
pub use self::ws::Sender;
pub use self::ws::Receiver;

pub mod ws;
pub mod client;
pub mod server;
pub mod dataframe;
pub mod message;
pub mod result;
pub mod stream;
pub mod header;
pub mod receiver;
pub mod sender;
