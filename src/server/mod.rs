//! Provides an implementation of a WebSocket server
use std::net::{SocketAddr, ToSocketAddrs, TcpListener, TcpStream};
use std::io;
use std::convert::Into;
#[cfg(feature="ssl")]
use openssl::ssl::{SslStream, SslAcceptor};
use stream::Stream;
use self::upgrade::{WsUpgrade, IntoWs, Buffer};
pub use self::upgrade::{Request, HyperIntoWsError};

pub mod upgrade;

/// When a sever tries to accept a connection many things can go wrong.
///
/// This struct is all the information that is recovered from a failed
/// websocket handshake, in case one wants to use the connection for something
/// else (such as HTTP).
pub struct InvalidConnection<S>
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
	pub buffer: Option<Buffer>,
	/// the cause of the failed websocket connection setup
	pub error: HyperIntoWsError,
}

/// Either the stream was established and it sent a websocket handshake
/// which represents the `Ok` variant, or there was an error (this is the
/// `Err` variant).
pub type AcceptResult<S> = Result<WsUpgrade<S>, InvalidConnection<S>>;

/// Marker struct for a struct not being secure
#[derive(Clone)]
pub struct NoSslAcceptor;
/// Trait that is implemented over NoSslAcceptor and SslAcceptor that
/// serves as a generic bound to make a struct with.
/// Used in the Server to specify impls based on wether the server
/// is running over SSL or not.
pub trait OptionalSslAcceptor: Clone {}
impl OptionalSslAcceptor for NoSslAcceptor {}
#[cfg(feature="ssl")]
impl OptionalSslAcceptor for SslAcceptor {}

/// Represents a WebSocket server which can work with either normal
/// (non-secure) connections, or secure WebSocket connections.
///
/// This is a convenient way to implement WebSocket servers, however
/// it is possible to use any sendable Reader and Writer to obtain
/// a WebSocketClient, so if needed, an alternative server implementation can be used.
///# Non-secure Servers
///
/// ```no_run
///extern crate websocket;
///# fn main() {
///use std::thread;
///use websocket::{Server, Message};
///
///let server = Server::bind("127.0.0.1:1234").unwrap();
///
///for connection in server.filter_map(Result::ok) {
///    // Spawn a new thread for each connection.
///    thread::spawn(move || {
///		   let mut client = connection.accept().unwrap();
///
///		   let message = Message::text("Hello, client!");
///		   let _ = client.send_message(&message);
///
///		   // ...
///    });
///}
/// # }
/// ```
///
///# Secure Servers
/// ```no_run
///extern crate websocket;
///extern crate openssl;
///# fn main() {
///use std::thread;
///use std::io::Read;
///use std::fs::File;
///use websocket::{Server, Message};
///use openssl::pkcs12::Pkcs12;
///use openssl::ssl::{SslMethod, SslAcceptorBuilder, SslStream};
///
///// In this example we retrieve our keypair and certificate chain from a PKCS #12 archive,
///// but but they can also be retrieved from, for example, individual PEM- or DER-formatted
///// files. See the documentation for the `PKey` and `X509` types for more details.
///let mut file = File::open("identity.pfx").unwrap();
///let mut pkcs12 = vec![];
///file.read_to_end(&mut pkcs12).unwrap();
///let pkcs12 = Pkcs12::from_der(&pkcs12).unwrap();
///let identity = pkcs12.parse("password123").unwrap();
///
///let acceptor = SslAcceptorBuilder::mozilla_intermediate(SslMethod::tls(),
///                                                        &identity.pkey,
///                                                        &identity.cert,
///                                                        &identity.chain)
///                   .unwrap()
///                   .build();
///
///let server = Server::bind_secure("127.0.0.1:1234", acceptor).unwrap();
///
///for connection in server.filter_map(Result::ok) {
///    // Spawn a new thread for each connection.
///    thread::spawn(move || {
///		   let mut client = connection.accept().unwrap();
///
///		   let message = Message::text("Hello, client!");
///		   let _ = client.send_message(&message);
///
///		   // ...
///    });
///}
/// # }
/// ```
///
/// # A Hyper Server
/// This crates comes with hyper integration out of the box, you can create a hyper
/// server and serve websocket and HTTP **on the same port!**
/// check out the docs over at `websocket::server::upgrade::from_hyper` for an example.
///
/// # A Custom Server
/// So you don't want to use any of our server implementations? That's O.K.
/// All it takes is implementing the `IntoWs` trait for your server's streams,
/// then calling `.into_ws()` on them.
/// check out the docs over at `websocket::server::upgrade` for more.
pub struct Server<S>
	where S: OptionalSslAcceptor
{
	listener: TcpListener,
	ssl_acceptor: S,
}

impl<S> Server<S>
    where S: OptionalSslAcceptor
{
	/// Get the socket address of this server
	pub fn local_addr(&self) -> io::Result<SocketAddr> {
		self.listener.local_addr()
	}

	/// Create a new independently owned handle to the underlying socket.
	pub fn try_clone(&self) -> io::Result<Server<S>> {
		let inner = try!(self.listener.try_clone());
		Ok(Server {
		       listener: inner,
		       ssl_acceptor: self.ssl_acceptor.clone(),
		   })
	}

	/// Changes whether the Server is in nonblocking mode.
	///
	/// If it is in nonblocking mode, accept() will return an error instead of blocking when there
	/// are no incoming connections.
	///
	///# Examples
	///```no_run
	/// # extern crate websocket;
	/// # use websocket::Server;
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
	/// 	let result = server.accept();
	/// 	match result {
	/// 		Ok(wsupgrade) => {
	/// 			// Do something with the established TcpStream.
	/// 		}
	/// 		_ => {
	/// 			// Nobody tried to connect, move on.
	/// 		}
	/// 	}
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
}

#[cfg(feature="ssl")]
impl Server<SslAcceptor> {
	/// Bind this Server to this socket, utilising the given SslContext
	pub fn bind_secure<A>(addr: A, acceptor: SslAcceptor) -> io::Result<Self>
		where A: ToSocketAddrs
	{
		Ok(Server {
		       listener: try!(TcpListener::bind(&addr)),
		       ssl_acceptor: acceptor,
		   })
	}

	/// Wait for and accept an incoming WebSocket connection, returning a WebSocketRequest
	pub fn accept(&mut self) -> AcceptResult<SslStream<TcpStream>> {
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

#[cfg(feature="ssl")]
impl Iterator for Server<SslAcceptor> {
	type Item = AcceptResult<SslStream<TcpStream>>;

	fn next(&mut self) -> Option<<Self as Iterator>::Item> {
		Some(self.accept())
	}
}

impl Server<NoSslAcceptor> {
	/// Bind this Server to this socket
	pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
		Ok(Server {
		       listener: try!(TcpListener::bind(&addr)),
		       ssl_acceptor: NoSslAcceptor,
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
}

impl Iterator for Server<NoSslAcceptor> {
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
		macro_rules! t {
			($e:expr) => {
				match $e {
					Ok(t) => t,
					Err(e) => panic!("received error for `{}`: {}", stringify!($e), e),
				}
			}
		}
		use super::*;

		// Test unsecure server

		let mut server = t!(Server::bind("127.0.0.1:0"));

		// Note that if set_nonblocking() doesn't work, but the following
		// fails to panic for some reason, then the .accept() method below
		// will block indefinitely.
		t!(server.set_nonblocking(true));

		let result = server.accept();
		match result {
			// nobody tried to establish a connection, so we expect an error
			Ok(_) => panic!("expected error"),
			Err(e) => {
				match e.error {
					HyperIntoWsError::Io(e) => if e.kind() == io::ErrorKind::WouldBlock {},
					_ => panic!("unexpected error {}"),
				}
			}
		}

	}
}
