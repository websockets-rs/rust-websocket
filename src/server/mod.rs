//! Provides an implementation of a WebSocket server
#[cfg(any(feature = "sync-ssl", feature = "async-ssl"))]
use native_tls::TlsAcceptor;

use self::upgrade::{HyperIntoWsError, Request};
use crate::stream::Stream;
use std::fmt::{Debug, Formatter, Result as FmtResult};

pub mod upgrade;

#[cfg(feature = "async")]
pub mod r#async;

#[cfg(feature = "sync")]
pub mod sync;

/// Marker struct for a struct not being secure
#[derive(Clone)]
pub struct NoTlsAcceptor;
/// Trait that is implemented over NoSslAcceptor and SslAcceptor that
/// serves as a generic bound to make a struct with.
/// Used in the Server to specify impls based on whether the server
/// is running over SSL or not.
pub trait OptionalTlsAcceptor {}
impl OptionalTlsAcceptor for NoTlsAcceptor {}
#[cfg(any(feature = "sync-ssl", feature = "async-ssl"))]
impl OptionalTlsAcceptor for TlsAcceptor {}

/// When a sever tries to accept a connection many things can go wrong.
///
/// This struct is all the information that is recovered from a failed
/// websocket handshake, in case one wants to use the connection for something
/// else (such as HTTP).
pub struct InvalidConnection<S, B>
where
	S: Stream,
{
	/// if the stream was successfully setup it will be included here
	/// on a failed connection.
	pub stream: Option<S>,
	/// the parsed request. **This is a normal HTTP request** meaning you can
	/// simply run this server and handle both HTTP and Websocket connections.
	/// If you already have a server you want to use, checkout the
	/// `server::upgrade` module to integrate this crate with your server.
	pub parsed: Option<Request>,
	/// the buffered data that was already taken from the stream
	pub buffer: Option<B>,
	/// the cause of the failed websocket connection setup
	pub error: HyperIntoWsError,
}

impl<S, B> Debug for InvalidConnection<S, B>
where
	S: Stream,
{
	fn fmt(&self, fmt: &mut Formatter<'_>) -> FmtResult {
		fmt.debug_struct("InvalidConnection")
			.field("stream", &String::from("..."))
			.field("parsed", &String::from("..."))
			.field("buffer", &String::from("..."))
			.field("error", &self.error)
			.finish()
	}
}

/// Represents a WebSocket server which can work with either normal
/// (non-secure) connections, or secure WebSocket connections.
///
/// This is a convenient way to implement WebSocket servers, however
/// it is possible to use any sendable Reader and Writer to obtain
/// a WebSocketClient, so if needed, an alternative server implementation can be used.
///
/// # Synchronous Servers
/// Synchronous implementations of a websocket server are available below, each method is
/// documented so the reader knows whether is it synchronous or asynchronous.
///
/// To use the synchronous implementation, you must have the `sync` feature enabled
/// (it is enabled by default).
/// To use the synchronous SSL implementation, you must have the `sync-ssl` feature enabled
/// (it is enabled by default).
///
/// # Asynchronous Servers
/// Asynchronous implementations of a websocket server are available below, each method is
/// documented so the reader knows whether is it synchronous or asynchronous.
/// Simply look out for the implementation of `Server` whose methods only return `Future`s
/// (it is also written in the docs if the method is async).
///
/// To use the asynchronous implementation, you must have the `async` feature enabled
/// (it is enabled by default).
/// To use the asynchronous SSL implementation, you must have the `async-ssl` feature enabled
/// (it is enabled by default).
///
/// # A Hyper Server
/// This crates comes with hyper integration out of the box, you can create a hyper
/// server and serve websocket and HTTP **on the same port!**
/// check out the docs over at `websocket::server::upgrade::sync::HyperRequest` for an example.
///
/// # A Custom Server
/// So you don't want to use any of our server implementations? That's O.K.
/// All it takes is implementing the `IntoWs` trait for your server's streams,
/// then calling `.into_ws()` on them.
/// check out the docs over at `websocket::server::upgrade::sync` for more.
#[cfg(any(feature = "sync", feature = "async"))]
pub struct WsServer<S, L>
where
	S: OptionalTlsAcceptor,
{
	listener: L,
	/// The SSL acceptor given to the server
	pub ssl_acceptor: S,
}
