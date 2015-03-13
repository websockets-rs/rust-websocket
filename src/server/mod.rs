//! Provides an implementation of a WebSocket server
use std::net::{SocketAddr, ToSocketAddrs, TcpListener};
use std::io;

pub use self::request::Request;
pub use self::response::Response;
pub use self::sender::Sender;
pub use self::receiver::Receiver;

use stream::WebSocketStream;

use openssl::ssl::SslContext;
use openssl::ssl::SslStream;

pub mod request;
pub mod response;
pub mod sender;
pub mod receiver;

/// Represents a WebSocket server which can work with either normal (non-secure) connections, or secure WebSocket connections.
///
/// This is a convenient way to implement WebSocket servers, however it is possible to use any sendable Reader and Writer to obtain
/// a WebSocketClient, so if needed, an alternative server implementation can be used.
///#Non-secure Servers
///
/// ```no_run
///extern crate websocket;
///# fn main() {
///use std::thread::Thread;
///use websocket::{Server, Message};
///
///let server = Server::bind("127.0.0.1:1234").unwrap();
///
///for request in server {
///    // Spawn a new thread for each connection.
///    Thread::spawn(move || {
///		   let request = request.unwrap(); // Get the request
///		   let response = request.accept(); // Form a response
///		   let mut client = response.send().unwrap(); // Send the response
///
///		   let message = Message::Text("Hello, client!".to_string());
///		   let _ = client.send_message(message);
///
///		   // ...
///    });
///}
/// # }
/// ```
///
///#Secure Servers
/// ```no_run
///extern crate websocket;
///extern crate openssl;
///# fn main() {
///use std::thread::Thread;
///use std::path::Path;
///use websocket::{Server, Message};
///use openssl::ssl::{SslContext, SslMethod};
///use openssl::x509::X509FileType;
///
///let mut context = SslContext::new(SslMethod::Tlsv1).unwrap();
///let _ = context.set_certificate_file(&(Path::new("cert.pem")), X509FileType::PEM);
///let _ = context.set_private_key_file(&(Path::new("key.pem")), X509FileType::PEM);
///let server = Server::bind_secure("127.0.0.1:1234", &context).unwrap();
///
///for request in server {
///    // Spawn a new thread for each connection.
///    Thread::spawn(move || {
///		   let request = request.unwrap(); // Get the request
///		   let response = request.accept(); // Form a response
///		   let mut client = response.send().unwrap(); // Send the response
///
///		   let message = Message::Text("Hello, client!".to_string());
///		   let _ = client.send_message(message);
///
///		   // ...
///    });
///}
/// # }
/// ```
pub struct Server<'a> {
	inner: TcpListener,
	context: Option<&'a SslContext>,
}

impl<'a> Server<'a> {
	/// Bind this Server to this socket
	pub fn bind<T: ToSocketAddrs>(addr: T) -> io::Result<Server<'a>> {
		Ok(Server {
			inner: try!(TcpListener::bind(&addr)),
			context: None,
		})
	}
	/// Bind this Server to this socket, utilising the given SslContext
	pub fn bind_secure<T: ToSocketAddrs>(addr: T, context: &'a SslContext) -> io::Result<Server<'a>> {
		Ok(Server {
			inner: try!(TcpListener::bind(&addr)),
			context: Some(context),
		})
	}
	/// Get the socket address of this server
	pub fn socket_addr(&mut self) -> io::Result<SocketAddr> {
		self.inner.socket_addr()
	}

	/// Prevents blocking on all future accepts after ms milliseconds have elapsed.
	///
	/// This function is used to set a deadline after which this acceptor will time out accepting any connections.
	/// The argument is the relative distance, in milliseconds, to a point in the future after which all accepts will fail.
	///
	/// If the argument specified is None, then any previously registered timeout is cleared.
	///
	/// A timeout of 0 can be used to "poll" this acceptor to see if it has any pending connections.
	/// All pending connections will be accepted, regardless of whether the timeout has expired or not (the accept will not block in this case).
	pub fn set_timeout(&mut self, ms: Option<u64>) {
		unimplemented!(); //Deadlines not yet implemented in new `TcpStream`
	}

	/// Wait for and accept an incoming WebSocket connection, returning a WebSocketRequest
	fn accept(&mut self) -> io::Result<Request<WebSocketStream, WebSocketStream>> {
		let stream = try!(self.inner.accept()).0;
		let wsstream = match self.context {
			Some(context) => {
				let sslstream = match SslStream::new_server(context, stream) {
					Ok(s) => s,
					Err(err) => {
						return Err(io::Error::new(
							io::ErrorKind::Other,
							"SSL Error",
							Some(format!("{:?}", err)),
						));
					}
				};
				WebSocketStream::Ssl(sslstream)
			}
			None => { WebSocketStream::Tcp(stream) }
		};

		match Request::read(try!(wsstream.try_clone()), try!(wsstream.try_clone())) {
			Ok(result) => { Ok(result) },
			Err(err) => {
				Err(io::Error::new(
					io::ErrorKind::InvalidInput,
					"Failed to read request",
					Some(format!("{:?}", err)),
				))
			}
		}
	}
}

impl<'a> Iterator for Server<'a> {
	type Item = io::Result<Request<WebSocketStream, WebSocketStream>>;

	fn next(&mut self) -> Option<<Self as Iterator>::Item> {
		Some(self.accept())
	}
}
