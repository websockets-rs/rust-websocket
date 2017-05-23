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
pub extern crate url;
extern crate rand;
extern crate byteorder;
extern crate sha1;
extern crate base64;
#[cfg(any(feature="sync-ssl", feature="async-ssl"))]
extern crate native_tls;
#[cfg(feature="async")]
extern crate tokio_core;
#[cfg(feature="async")]
extern crate tokio_io;
#[cfg(feature="async")]
extern crate bytes;
#[cfg(feature="async")]
extern crate futures;
#[cfg(feature="async-ssl")]
extern crate tokio_tls;

#[macro_use]
extern crate bitflags;

#[cfg(all(feature = "nightly", test))]
extern crate test;

macro_rules! upsert_header {
    ($headers:expr; $header:ty; {
        Some($pat:pat) => $some_match:expr,
        None => $default:expr
    }) => {{
        if $headers.has::<$header>() {
            if let Some($pat) = $headers.get_mut::<$header>() {
                $some_match
            }
        } else {
            $headers.set($default);
        }
    }}
}

pub mod ws;
pub mod dataframe;
pub mod message;
pub mod result;
pub mod header;

#[cfg(feature="async")]
pub mod codec;

#[cfg(feature="sync")]
pub mod receiver;
#[cfg(feature="sync")]
pub mod sender;

pub mod client;
pub mod server;
pub mod stream;

#[cfg(feature="sync")]
pub mod sync {
    pub use sender;
    pub use sender::Writer;

    pub use receiver;
    pub use receiver::Reader;

    pub use stream::sync::Stream;
    pub use stream::sync as stream;

    pub mod server {
        pub use server::sync::*;
        pub use server::upgrade::sync::Upgrade;
        pub use server::upgrade::sync::IntoWs;
        pub use server::upgrade::sync as upgrade;
    }

    pub mod client {
        pub use client::sync::*;
        pub use client::builder::ClientBuilder;
    }
}

#[cfg(feature="async")]
pub mod async {
    pub use codec;
    pub use codec::ws::MessageCodec;
    pub use codec::ws::Context as MessageContext;
    pub use codec::http::HttpClientCodec;
    pub use codec::http::HttpServerCodec;

    pub use stream::async::Stream;
    pub use stream::async as stream;

    pub mod server {
        pub use server::async::*;
        pub use server::upgrade::async::Upgrade;
        pub use server::upgrade::async::IntoWs;
        pub use server::upgrade::async as upgrade;
    }

    pub mod client {
        pub use client::async::*;
        pub use client::builder::ClientBuilder;
    }
}

pub use self::message::Message;
pub use self::message::CloseData;
pub use self::message::OwnedMessage;

pub use self::result::WebSocketError;
pub use self::result::WebSocketResult;

