#![warn(missing_docs)]
#![cfg_attr(all(test, feature = "nightly"), feature(test))]
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
extern crate base64;
extern crate byteorder;
#[cfg(feature = "async")]
extern crate bytes;
#[cfg(feature = "async")]
pub extern crate futures;
extern crate hyper;
#[cfg(any(feature = "sync-ssl", feature = "async-ssl"))]
extern crate native_tls;
extern crate rand;
extern crate sha1;
#[cfg(feature = "async")]
extern crate tokio_core;
#[cfg(feature = "async")]
extern crate tokio_io;
#[cfg(feature = "async-ssl")]
extern crate tokio_tls;
extern crate unicase;
pub extern crate url;

#[macro_use]
extern crate bitflags;

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

pub mod dataframe;
pub mod header;
pub mod message;
pub mod result;
pub mod ws;

#[cfg(feature = "async")]
pub mod codec;

#[cfg(feature = "sync")]
pub mod receiver;
#[cfg(feature = "sync")]
pub mod sender;

pub mod client;
pub mod server;
pub mod stream;

/// A collection of handy synchronous-only parts of the crate.
#[cfg(feature = "sync")]
pub mod sync {
	pub use sender;
	pub use sender::Writer;

	pub use receiver;
	pub use receiver::Reader;

	pub use stream::sync as stream;
	pub use stream::sync::Stream;

	/// A collection of handy synchronous-only parts of the `server` module.
	pub mod server {
		pub use server::sync::*;
		pub use server::upgrade::sync as upgrade;
		pub use server::upgrade::sync::IntoWs;
		pub use server::upgrade::sync::Upgrade;
	}
	pub use server::sync::Server;

	/// A collection of handy synchronous-only parts of the `client` module.
	pub mod client {
		pub use client::builder::ClientBuilder;
		pub use client::sync::*;
	}
	pub use client::sync::Client;
}

/// A collection of handy asynchronous-only parts of the crate.
#[cfg(feature = "async")]
pub mod async {
	pub use codec;
	pub use codec::http::HttpClientCodec;
	pub use codec::http::HttpServerCodec;
	pub use codec::ws::Context as MsgCodecCtx;
	pub use codec::ws::MessageCodec;

	pub use stream::async as stream;
	pub use stream::async::Stream;

	/// A collection of handy asynchronous-only parts of the `server` module.
	pub mod server {
		pub use server::async::*;
		pub use server::upgrade::async as upgrade;
		pub use server::upgrade::async::IntoWs;
		pub use server::upgrade::async::Upgrade;
	}
	pub use server::async::Server;

	/// A collection of handy asynchronous-only parts of the `client` module.
	pub mod client {
		pub use client::async::*;
		pub use client::builder::ClientBuilder;
	}
	pub use client::async::Client;

	pub use result::async::WebSocketFuture;

	pub use futures;
	pub use tokio_core::net::TcpListener;
	pub use tokio_core::net::TcpStream;
	pub use tokio_core::reactor::Core;
	pub use tokio_core::reactor::Handle;
}

pub use self::client::builder::ClientBuilder;
pub use self::message::CloseData;
pub use self::message::Message;
pub use self::message::OwnedMessage;

pub use self::result::WebSocketError;
pub use self::result::WebSocketResult;
