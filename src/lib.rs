#![cfg_attr(all(test, feature = "nightly"), feature(test))]
#![allow(
	clippy::write_with_newline,
	clippy::type_complexity,
	clippy::match_ref_pats,
	clippy::needless_doctest_main
)]
#![deny(unused_mut)]

//! Rust-WebSocket is a WebSocket (RFC6455) library written in Rust.
//!
//! # Synchronous and Asynchronous
//! This crate has both async and sync implementations of websockets, you are free to
//! choose which one you would like to use by switching on the `async` or `sync` features
//! for this crate. By default both are switched on since they do not conflict with each
//! other.
//!
//! You'll find many modules with `::sync` and `::async` submodules that separate these
//! behaviours. Since it get's tedious to add these on when appropriate a top-level
//! convenience module called `websocket::sync` and `websocket::async` has been added that
//! groups all the sync and async stuff, respectively.
//!
//! # Clients
//! To make a client use the `ClientBuilder` struct, this builder has methods
//! for creating both synchronous and asynchronous clients.
//!
//! # Servers
//! WebSocket servers act similarly to the `TcpListener`, and listen for connections.
//! See the `Server` struct documentation for more information. The `bind()` and
//! `bind_secure()` functions will bind the server to the given `SocketAddr`.
//!
//! # Extending Rust-WebSocket
//! The `ws` module contains the traits and functions used by Rust-WebSocket at a lower
//! level. Their usage is explained in the module documentation.

extern crate hyper;
#[cfg(test)]
extern crate tokio;

extern crate unicase;
pub extern crate url;

pub extern crate websocket_base;

#[cfg(all(feature = "nightly", test))]
extern crate test;

macro_rules! upsert_header {
	($headers:expr; $header:ty;  { Some($pat:pat) => $some_match:expr,None => $default:expr }) => {{
		if $headers.has::<$header>() {
			if let Some($pat) = $headers.get_mut::<$header>() {
				$some_match
			}
		} else {
			$headers.set($default);
		}
	}};
}

pub use websocket_base::dataframe;
pub mod header;
pub use websocket_base::message;
pub mod result;
pub use websocket_base::ws;
pub mod receiver;
pub mod sender;

pub mod client;
pub mod server;
pub use websocket_base::stream;

pub mod sync {
	pub use crate::sender;
	pub use crate::sender::Writer;

	pub use crate::receiver;
	pub use crate::receiver::Reader;

	pub use crate::stream::sync as stream;
	pub use crate::stream::sync::Stream;

	/// A collection of handy synchronous-only parts of the `server` module.
	pub mod server {
		pub use crate::server::sync::*;
		pub use crate::server::upgrade::sync as upgrade;
		pub use crate::server::upgrade::sync::IntoWs;
		pub use crate::server::upgrade::sync::Upgrade;
	}
	pub use crate::server::sync::Server;

	/// A collection of handy synchronous-only parts of the `client` module.
	pub mod client {
		pub use crate::client::builder::ClientBuilder;
		pub use crate::client::sync::*;
	}
	pub use crate::client::sync::Client;
}

pub use self::client::builder::ClientBuilder;
pub use self::message::CloseData;
pub use self::message::Message;
pub use self::message::OwnedMessage;

pub use self::result::WebSocketError;
pub use self::result::WebSocketResult;
