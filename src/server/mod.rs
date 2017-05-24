//! Provides an implementation of a WebSocket server
#[cfg(any(feature="sync-ssl", feature="async-ssl"))]
use native_tls::TlsAcceptor;

use stream::Stream;
use self::upgrade::{Request, HyperIntoWsError};

pub mod upgrade;

#[cfg(feature="async")]
pub mod async;

#[cfg(feature="sync")]
pub mod sync;

/// Marker struct for a struct not being secure
#[derive(Clone)]
pub struct NoTlsAcceptor;
/// Trait that is implemented over NoSslAcceptor and SslAcceptor that
/// serves as a generic bound to make a struct with.
/// Used in the Server to specify impls based on wether the server
/// is running over SSL or not.
pub trait OptionalTlsAcceptor {}
impl OptionalTlsAcceptor for NoTlsAcceptor {}
#[cfg(any(feature="sync-ssl", feature="async-ssl"))]
impl OptionalTlsAcceptor for TlsAcceptor {}

/// When a sever tries to accept a connection many things can go wrong.
///
/// This struct is all the information that is recovered from a failed
/// websocket handshake, in case one wants to use the connection for something
/// else (such as HTTP).
pub struct InvalidConnection<S, B>
	where S: Stream
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

pub struct WsServer<S, L>
	where S: OptionalTlsAcceptor
{
	listener: L,
	ssl_acceptor: S,
}
