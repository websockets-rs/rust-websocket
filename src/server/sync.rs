//! Provides an implementation of a WebSocket server
use std::net::{SocketAddr, ToSocketAddrs, TcpListener, TcpStream};
use std::io;
use std::convert::Into;
#[cfg(feature="sync-ssl")]
use native_tls::{TlsStream, TlsAcceptor};
use server::{WsServer, OptionalTlsAcceptor, NoTlsAcceptor, InvalidConnection};
use server::upgrade::sync::{Upgrade, IntoWs, Buffer};
pub use server::upgrade::{Request, HyperIntoWsError};

#[cfg(feature="async")]
use tokio_core::reactor::Handle;
#[cfg(feature="async")]
use tokio_core::net::TcpListener as AsyncTcpListener;
#[cfg(feature="async")]
use server::async;

/// Either the stream was established and it sent a websocket handshake
/// which represents the `Ok` variant, or there was an error (this is the
/// `Err` variant).
pub type AcceptResult<S> = Result<Upgrade<S>, InvalidConnection<S, Buffer>>;

/// Represents a WebSocket server which can work with either normal
/// (non-secure) connections, or secure WebSocket connections.
///
/// This is a convenient way to implement WebSocket servers, however
/// it is possible to use any sendable Reader and Writer to obtain
/// a WebSocketClient, so if needed, an alternative server implementation can be used.
pub type Server<S> = WsServer<S, TcpListener>;

/// Synchronous methods for creating a server and accepting incoming connections.
impl<S> WsServer<S, TcpListener>
    where S: OptionalTlsAcceptor
{
	/// Get the socket address of this server
	pub fn local_addr(&self) -> io::Result<SocketAddr> {
		self.listener.local_addr()
	}

	/// Changes whether the Server is in nonblocking mode.
	/// NOTE: It is strongly encouraged to use the `websocket::async` module instead
	/// of this. It provides high level APIs for creating asynchronous servers.
	///
	/// If it is in nonblocking mode, accept() will return an error instead of
	/// blocking when there are no incoming connections.
	///
	///```no_run
	/// # extern crate websocket;
	/// # use websocket::sync::Server;
	/// # fn main() {
	/// // Suppose we have to work in a single thread, but want to
	/// // accomplish two unrelated things:
	/// // (1) Once in a while we want to check if anybody tried to connect to
	/// // our websocket server, and if so, handle the TcpStream.
	/// // (2) In between we need to something else, possibly unrelated to networking.
	///
	/// let mut server = Server::bind("127.0.0.1:0").unwrap();
	///
	/// // Set the server to non-blocking.
	/// server.set_nonblocking(true);
	///
	/// for i in 1..3 {
	/// 	let result = match server.accept() {
	/// 		Ok(wsupgrade) => {
	/// 			// Do something with the established TcpStream.
	/// 		}
	/// 		_ => {
	/// 			// Nobody tried to connect, move on.
	/// 		}
	/// 	};
	/// 	// Perform another task. Because we have a non-blocking server,
	/// 	// this will execute independent of whether someone tried to
	/// 	// establish a connection.
	/// 	let two = 1+1;
	/// }
	/// # }
	///```
	pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
		self.listener.set_nonblocking(nonblocking)
	}

	/// Turns an existing synchronous server into an asynchronous one.
	/// This will only work if the stream used for this server `S` already implements
	/// `AsyncRead + AsyncWrite`. Useful if you would like some blocking things to happen
	/// at the start of your server.
	#[cfg(feature="async")]
	pub fn into_async(self, handle: &Handle) -> io::Result<async::Server<S>> {
		let addr = self.listener.local_addr()?;
		Ok(WsServer {
		       listener: AsyncTcpListener::from_listener(self.listener, &addr, handle)?,
		       ssl_acceptor: self.ssl_acceptor,
		   })
	}
}

/// Synchronous methods for creating an SSL server and accepting incoming connections.
#[cfg(feature="sync-ssl")]
impl WsServer<TlsAcceptor, TcpListener> {
	/// Bind this Server to this socket, utilising the given SslContext
	///
	/// # Secure Servers
	/// ```no_run
	/// extern crate websocket;
	/// extern crate native_tls;
	/// # fn main() {
	/// use std::thread;
	/// use std::io::Read;
	/// use std::fs::File;
	/// use websocket::Message;
	/// use websocket::sync::Server;
	/// use native_tls::{Pkcs12, TlsAcceptor};
	///
	/// // In this example we retrieve our keypair and certificate chain from a PKCS #12 archive,
	/// // but but they can also be retrieved from, for example, individual PEM- or DER-formatted
	/// // files. See the documentation for the `PKey` and `X509` types for more details.
	/// let mut file = File::open("identity.pfx").unwrap();
	/// let mut pkcs12 = vec![];
	/// file.read_to_end(&mut pkcs12).unwrap();
	/// let pkcs12 = Pkcs12::from_der(&pkcs12, "hacktheplanet").unwrap();
	///
	/// let acceptor = TlsAcceptor::builder(pkcs12).unwrap().build().unwrap();
	///
	/// let server = Server::bind_secure("127.0.0.1:1234", acceptor).unwrap();
	///
	/// for connection in server.filter_map(Result::ok) {
	///     // Spawn a new thread for each connection.
	///     thread::spawn(move || {
	/// 		    let mut client = connection.accept().unwrap();
	///
	/// 		    let message = Message::text("Hello, client!");
	/// 		    let _ = client.send_message(&message);
	///
	/// 		    // ...
	///     });
	/// }
	/// # }
	/// ```
	pub fn bind_secure<A>(addr: A, acceptor: TlsAcceptor) -> io::Result<Self>
		where A: ToSocketAddrs
	{
		Ok(Server {
		       listener: TcpListener::bind(&addr)?,
		       ssl_acceptor: acceptor,
		   })
	}

	/// Wait for and accept an incoming WebSocket connection, returning a WebSocketRequest
	pub fn accept(&mut self) -> AcceptResult<TlsStream<TcpStream>> {
		let stream = match self.listener.accept() {
			Ok(s) => s.0,
			Err(e) => {
				return Err(InvalidConnection {
				               stream: None,
				               parsed: None,
				               buffer: None,
				               error: e.into(),
				           })
			}
		};

		let stream = match self.ssl_acceptor.accept(stream) {
			Ok(s) => s,
			Err(err) => {
				return Err(InvalidConnection {
				               stream: None,
				               parsed: None,
				               buffer: None,
				               error: io::Error::new(io::ErrorKind::Other, err).into(),
				           })
			}
		};

		match stream.into_ws() {
			Ok(u) => Ok(u),
			Err((s, r, b, e)) => {
				Err(InvalidConnection {
				        stream: Some(s),
				        parsed: r,
				        buffer: b,
				        error: e.into(),
				    })
			}
		}
	}
}

#[cfg(feature="sync-ssl")]
impl Iterator for WsServer<TlsAcceptor, TcpListener> {
	type Item = AcceptResult<TlsStream<TcpStream>>;

	fn next(&mut self) -> Option<<Self as Iterator>::Item> {
		Some(self.accept())
	}
}

impl WsServer<NoTlsAcceptor, TcpListener> {
	/// Bind this Server to this socket
	///
	/// # Non-secure Servers
	///
	/// ```no_run
	/// extern crate websocket;
	/// # fn main() {
	/// use std::thread;
	/// use websocket::Message;
	/// use websocket::sync::Server;
	///
	/// let server = Server::bind("127.0.0.1:1234").unwrap();
	///
	/// for connection in server.filter_map(Result::ok) {
	///     // Spawn a new thread for each connection.
	///     thread::spawn(move || {
	///		      let mut client = connection.accept().unwrap();
	///
	///		      let message = Message::text("Hello, client!");
	///		      let _ = client.send_message(&message);
	///
	///		      // ...
	///    });
	/// }
	/// # }
	/// ```
	pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
		Ok(Server {
		       listener: TcpListener::bind(&addr)?,
		       ssl_acceptor: NoTlsAcceptor,
		   })
	}

	/// Wait for and accept an incoming WebSocket connection, returning a WebSocketRequest
	pub fn accept(&mut self) -> AcceptResult<TcpStream> {
		let stream = match self.listener.accept() {
			Ok(s) => s.0,
			Err(e) => {
				return Err(InvalidConnection {
				               stream: None,
				               parsed: None,
				               buffer: None,
				               error: e.into(),
				           })
			}
		};

		match stream.into_ws() {
			Ok(u) => Ok(u),
			Err((s, r, b, e)) => {
				Err(InvalidConnection {
				        stream: Some(s),
				        parsed: r,
				        buffer: b,
				        error: e.into(),
				    })
			}
		}
	}

	/// Create a new independently owned handle to the underlying socket.
	pub fn try_clone(&self) -> io::Result<Self> {
		let inner = self.listener.try_clone()?;
		Ok(Server {
		       listener: inner,
		       ssl_acceptor: self.ssl_acceptor.clone(),
		   })
	}
}

impl Iterator for WsServer<NoTlsAcceptor, TcpListener> {
	type Item = AcceptResult<TcpStream>;

	fn next(&mut self) -> Option<<Self as Iterator>::Item> {
		Some(self.accept())
	}
}

mod tests {
	#[test]
	// test the set_nonblocking() method for Server<NoSslAcceptor>.
	// Some of this is copied from
	// https://doc.rust-lang.org/src/std/net/tcp.rs.html#1413
	fn set_nonblocking() {

		use super::*;

		// Test unsecure server

		let mut server = Server::bind("127.0.0.1:0").unwrap();

		// Note that if set_nonblocking() doesn't work, but the following
		// fails to panic for some reason, then the .accept() method below
		// will block indefinitely.
		server.set_nonblocking(true).unwrap();

		let result = server.accept();
		match result {
			// nobody tried to establish a connection, so we expect an error
			Ok(_) => panic!("expected error"),
			Err(e) => {
				match e.error {
					HyperIntoWsError::Io(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
					_ => panic!("unexpected error {}"),
				}
			}
		}

	}
}
