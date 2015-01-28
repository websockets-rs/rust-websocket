//! Provides an implementation of a WebSocket server
use std::old_io::{Listener, Acceptor};
use std::old_io::net::tcp::{TcpListener, TcpAcceptor};
use std::old_io::net::ip::{SocketAddr, ToSocketAddr};
use std::old_io::{IoResult, IoError, IoErrorKind};

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
///use std::old_io::{Listener, Acceptor};
///use websocket::{Server, Message};
///
///let server = Server::bind("127.0.0.1:1234").unwrap();
///
///let mut acceptor = server.listen().unwrap();
///for request in acceptor.incoming() {
///    // Spawn a new thread for each connection.
///    Thread::spawn(move || {
///        let request = request.unwrap(); // Get the request
///        let response = request.accept(); // Form a response
///        let mut client = response.send().unwrap(); // Send the response
///        
///        let message = Message::Text("Hello, client!".to_string());
///        let _ = client.send_message(message);
///        
///        // ...
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
///use std::old_io::{Listener, Acceptor};
///use websocket::{Server, Message};
///use openssl::ssl::{SslContext, SslMethod};
///use openssl::x509::X509FileType;
///
///let mut context = SslContext::new(SslMethod::Tlsv1).unwrap();
///let _ = context.set_certificate_file(&(Path::new("cert.pem")), X509FileType::PEM);
///let _ = context.set_private_key_file(&(Path::new("key.pem")), X509FileType::PEM);
///let server = Server::bind_secure("127.0.0.1:1234", &context).unwrap();
///
///let mut acceptor = server.listen().unwrap();
///for request in acceptor.incoming() {
///    // Spawn a new thread for each connection.
///    Thread::spawn(move || {
///        let request = request.unwrap(); // Get the request
///        let response = request.accept(); // Form a response
///        let mut client = response.send().unwrap(); // Send the response
///        
///        let message = Message::Text("Hello, client!".to_string());
///        let _ = client.send_message(message);
///        
///        // ...
///    });
///}
/// # }
/// ```
pub struct Server<'a> {
	inner: TcpListener,
	context: Option<&'a SslContext>,
}

/// An Acceptor for WebSocket connections
pub struct WebSocketAcceptor<'a> {
    inner: TcpAcceptor,
	context: Option<&'a SslContext>,
}

impl<'a> Server<'a> {
	/// Bind this Server to this socket
	pub fn bind<T: ToSocketAddr>(addr: T) -> IoResult<Server<'a>> {
		Ok(Server {
			inner: try!(TcpListener::bind(addr)),
			context: None,
		})
	}
	/// Bind this Server to this socket, utilising the given SslContext
	pub fn bind_secure<T: ToSocketAddr>(addr: T, context: &'a SslContext) -> IoResult<Server<'a>> {
		Ok(Server {
			inner: try!(TcpListener::bind(addr)),
			context: Some(context),
		})
	}
	/// Get the socket name of this server
	pub fn socket_name(&mut self) -> IoResult<SocketAddr> {
		self.inner.socket_name()
    }
}

impl<'a> Listener<Request<WebSocketStream, WebSocketStream>, WebSocketAcceptor<'a>> for Server<'a> {
	/// Begin listening for connections
	fn listen(self) -> IoResult<WebSocketAcceptor<'a>> {
		Ok(WebSocketAcceptor {
			inner: try!(self.inner.listen()),
			context: self.context,
		})
	}
}

impl<'a> WebSocketAcceptor<'a> {
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
		self.inner.set_timeout(ms)
	}
	/// Closes the accepting capabilities of this acceptor.
	/// 
	/// Once this function succeeds, all future calls to accept will return immediately with an error,
	/// preventing all future calls to accept. The underlying socket will not be relinquished back to
	/// the OS until all acceptors have been deallocated.
	/// 
	/// This is useful for waking up a thread in an accept loop to indicate that it should exit.
	pub fn close_accept(&mut self) -> IoResult<()> {
		self.inner.close_accept()
	}
}

impl<'a> Acceptor<Request<WebSocketStream, WebSocketStream>> for WebSocketAcceptor<'a> {
	/// Wait for and accept an incoming WebSocket connection, returning a WebSocketRequest
    fn accept(&mut self) -> IoResult<Request<WebSocketStream, WebSocketStream>> {
        let stream = try!(self.inner.accept());
		let wsstream = match self.context {
			Some(context) => {
				let sslstream = match SslStream::new_server(context, stream) {
					Ok(s) => s,
					Err(err) => {
						return Err(IoError {
							kind: IoErrorKind::OtherIoError,
							desc: "SSL Error",
							detail: Some(format!("{:?}", err)),
						});
					}
				};
				WebSocketStream::Ssl(sslstream)
			}
			None => { WebSocketStream::Tcp(stream) }
		};
		
		match Request::read(wsstream.clone(), wsstream.clone()) {
			Ok(result) => { Ok(result) },
			Err(err) => {
				Err(IoError {
					kind: IoErrorKind::InvalidInput,
					desc: "Failed to read request",
					detail: Some(format!("{:?}", err)),
				})
			}
		}
    }
}

impl<'a> Clone for WebSocketAcceptor<'a> {
	fn clone(&self) -> Self {
		WebSocketAcceptor {
			inner: self.inner.clone(),
			context: self.context
		}
	}
}
