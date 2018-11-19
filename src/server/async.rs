//! The asynchronous implementation of a websocket server.
use bytes::BytesMut;
use futures::{Future, Stream};
use server::upgrade::async::{IntoWs, Upgrade};
use server::InvalidConnection;
use server::{NoTlsAcceptor, WsServer};
use std::io;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use tokio::net::{TcpListener, TcpStream};
pub use tokio::reactor::Handle;

#[cfg(any(feature = "async-ssl"))]
use native_tls::TlsAcceptor;
#[cfg(any(feature = "async-ssl"))]
use tokio_tls::{TlsAcceptor as TlsAcceptorExt, TlsStream};

/// The asynchronous specialization of a websocket server.
/// Use this struct to create asynchronous servers.
pub type Server<S> = WsServer<S, TcpListener>;

/// A stream of websocket connections and addresses the server generates.
///
/// Each item of the stream is the address of the incoming connection and an `Upgrade`
/// struct which lets the user decide whether to turn the connection into a websocket
/// connection or reject it.
pub type Incoming<S> =
	Box<Stream<Item = (Upgrade<S>, SocketAddr), Error = InvalidConnection<S, BytesMut>> + Send>;

/// Asynchronous methods for creating an async server and accepting incoming connections.
impl WsServer<NoTlsAcceptor, TcpListener> {
	/// Bind a websocket server to an address.
	/// Creating a websocket server can be done immediately so this does not
	/// return a `Future` but a simple `Result`.
	pub fn bind<A: ToSocketAddrs>(addr: A, handle: &Handle) -> io::Result<Self> {
		let tcp = ::std::net::TcpListener::bind(addr)?;
		Ok(Server {
			listener: TcpListener::from_std(tcp, handle)?,
			ssl_acceptor: NoTlsAcceptor,
		})
	}

	/// Turns the server into a stream of connection objects.
	///
	/// Each item of the stream is the address of the incoming connection and an `Upgrade`
	/// struct which lets the user decide whether to turn the connection into a websocket
	/// connection or reject it.
	///
	/// See the [`examples/async-server.rs`]
	/// (https://github.com/cyderize/rust-websocket/blob/master/examples/async-server.rs)
	/// example for a good echo server example.
	pub fn incoming(self) -> Incoming<TcpStream> {
		let future = self
			.listener
			.incoming()
			.and_then(|s| s.peer_addr().map(|a| (s, a)))
			.map_err(|e| InvalidConnection {
				stream: None,
				parsed: None,
				buffer: None,
				error: e.into(),
			})
			.and_then(|(stream, a)| {
				stream
					.into_ws()
					.map_err(|(stream, req, buf, err)| InvalidConnection {
						stream: Some(stream),
						parsed: req,
						buffer: Some(buf),
						error: err,
					})
					.map(move |u| (u, a))
			});
		Box::new(future)
	}
}

/// Asynchronous methods for creating an async SSL server and accepting incoming connections.
#[cfg(any(feature = "async-ssl"))]
impl WsServer<TlsAcceptor, TcpListener> {
	/// Bind an SSL websocket server to an address.
	/// Creating a websocket server can be done immediately so this does not
	/// return a `Future` but a simple `Result`.
	///
	/// Since this is an SSL server one needs to provide a `TlsAcceptor` that contains
	/// the server's SSL information.
	pub fn bind_secure<A: ToSocketAddrs>(
		addr: A,
		acceptor: TlsAcceptor,
		handle: &Handle,
	) -> io::Result<Self> {
		let tcp = ::std::net::TcpListener::bind(addr)?;
		Ok(Server {
			listener: TcpListener::from_std(tcp, handle)?,
			ssl_acceptor: acceptor,
		})
	}

	/// Turns the server into a stream of connection objects.
	///
	/// Each item of the stream is the address of the incoming connection and an `Upgrade`
	/// struct which lets the user decide whether to turn the connection into a websocket
	/// connection or reject it.
	///
	/// See the [`examples/async-server.rs`]
	/// (https://github.com/cyderize/rust-websocket/blob/master/examples/async-server.rs)
	/// example for a good echo server example.
	pub fn incoming(self) -> Incoming<TlsStream<TcpStream>> {
		let acceptor = TlsAcceptorExt::from(self.ssl_acceptor);
		let future = self
			.listener
			.incoming()
			.and_then(|s| s.peer_addr().map(|a| (s, a)))
			.map_err(|e| InvalidConnection {
				stream: None,
				parsed: None,
				buffer: None,
				error: e.into(),
			})
			.and_then(move |(stream, a)| {
				acceptor
					.accept(stream)
					.map_err(|e| {
						InvalidConnection {
							stream: None,
							parsed: None,
							buffer: None,
							// TODO: better error types
							error: io::Error::new(io::ErrorKind::Other, e).into(),
						}
					})
					.map(move |s| (s, a))
			})
			.and_then(|(stream, a)| {
				stream
					.into_ws()
					.map_err(|(stream, req, buf, err)| InvalidConnection {
						stream: Some(stream),
						parsed: req,
						buffer: Some(buf),
						error: err,
					})
					.map(move |u| (u, a))
			});
		Box::new(future)
	}
}
