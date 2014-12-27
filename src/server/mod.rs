//! Provides an implementation of a WebSocket server
#![unstable]
use std::io::{Listener, Acceptor};
use std::io::net::tcp::{TcpListener, TcpAcceptor};
use std::io::net::ip::{SocketAddr, ToSocketAddr};
use std::io::{IoResult, IoError, IoErrorKind};
use handshake::WebSocketRequest;
use common::{Inbound, WebSocketStream};
use openssl::ssl::SslContext;
use openssl::ssl::SslStream;

/// Represents a WebSocketServer which can work with either normal (non-secure) connections, or secure WebSocket connections.
///
/// This is a convenient way to implement WebSocket servers, however it is possible to use any sendable Reader and Writer to obtain
/// a WebSocketClient, so if needed, an alternative server implementation can be used. 
///#Non-secure Servers
///
/// ```no_run
///extern crate websocket;
///# fn main() {
///use std::thread::Thread;
///use std::io::{Listener, Acceptor};
///use websocket::{WebSocketServer, WebSocketMessage};
///
///let server = WebSocketServer::bind("127.0.0.1:1234").unwrap();
///
///let mut acceptor = server.listen().unwrap();
///for request in acceptor.incoming() {
///    // Spawn a new thread for each connection.
///    Thread::spawn(move || {
///        let request = request.unwrap(); // Get the request
///        let response = request.accept(); // Form a response
///        let mut client = response.send().unwrap(); // Send the response
///        
///        let message = WebSocketMessage::Text("Hello, client!".to_string());
///        let _ = client.send_message(message);
///        
///        // ...
///    }).detach();
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
///use std::io::{Listener, Acceptor};
///use websocket::{WebSocketServer, WebSocketMessage};
///use openssl::ssl::{SslContext, SslMethod};
///use openssl::x509::X509FileType;
///
///let mut context = SslContext::new(SslMethod::Tlsv1).unwrap();
///let _ = context.set_certificate_file(&(Path::new("cert.pem")), X509FileType::PEM);
///let _ = context.set_private_key_file(&(Path::new("key.pem")), X509FileType::PEM);
///let server = WebSocketServer::bind_secure("127.0.0.1:1234", &context).unwrap();
///
///let mut acceptor = server.listen().unwrap();
///for request in acceptor.incoming() {
///    // Spawn a new thread for each connection.
///    Thread::spawn(move || {
///        let request = request.unwrap(); // Get the request
///        let response = request.accept(); // Form a response
///        let mut client = response.send().unwrap(); // Send the response
///        
///        let message = WebSocketMessage::Text("Hello, client!".to_string());
///        let _ = client.send_message(message);
///        
///        // ...
///    }).detach();
///}
/// # }
/// ```
///#Custom implementations
/// Rust-WebSocket allows you to use any (sendable) Reader and Writer with a WebSocketClient.
/// This means you can implement your own server in which ever way you please.
/// 
/// The essential things are to obtain a Reader and a Writer and pass them to ```WebSocketRequest::read()```.
/// This will give you an inbound WebSocketRequest which can be used with ```WebSocketResponse::new()```.
/// Finally, call ```begin()``` or ```begin_with()``` on the response to obtain a WebSocketClient and begin
/// sending/receiving messages.
#[unstable]
pub struct WebSocketServer<'a> {
	inner: TcpListener,
	context: Option<&'a SslContext>,
}

/// An Acceptor for WebSocket connections
pub struct WebSocketAcceptor<'a> {
    inner: TcpAcceptor,
	context: Option<&'a SslContext>,
}

impl<'a> WebSocketServer<'a> {
	/// Bind this WebSocketServer to this socket
	pub fn bind<T: ToSocketAddr>(addr: T) -> IoResult<WebSocketServer<'a>> {
		Ok(WebSocketServer {
			inner: try!(TcpListener::bind(addr)),
			context: None,
		})
	}
	/// Bind this WebSocketServer to this socket, utilising the given SslContext
	pub fn bind_secure<T: ToSocketAddr>(addr: T, context: &'a SslContext) -> IoResult<WebSocketServer<'a>> {
		Ok(WebSocketServer {
			inner: try!(TcpListener::bind(addr)),
			context: Some(context),
		})
	}
	/// Get the socket name of this server
	pub fn socket_name(&mut self) -> IoResult<SocketAddr> {
		self.inner.socket_name()
    }
}

impl<'a> Listener<WebSocketRequest<WebSocketStream, WebSocketStream, Inbound>, WebSocketAcceptor<'a>> for WebSocketServer<'a> {
	/// Begin listening for connections
	fn listen(self) -> IoResult<WebSocketAcceptor<'a>> {
		Ok(WebSocketAcceptor {
			inner: try!(self.inner.listen()),
			context: self.context,
		})
	}
}

impl<'a> Acceptor<WebSocketRequest<WebSocketStream, WebSocketStream, Inbound>> for WebSocketAcceptor<'a> {
	/// Wait for and accept an incoming WebSocket connection, returning a WebSocketRequest
    fn accept(&mut self) -> IoResult<WebSocketRequest<WebSocketStream, WebSocketStream, Inbound>> {
        let stream = try!(self.inner.accept());
		let wsstream = match self.context {
			Some(context) => {
				let sslstream = match SslStream::new_server(context, stream) {
					Ok(s) => s,
					Err(err) => {
						return Err(IoError {
							kind: IoErrorKind::OtherIoError,
							desc: "SSL Error",
							detail: Some(err.to_string()),
						});
					}
				};
				WebSocketStream::Secure(sslstream)
			}
			None => { WebSocketStream::Normal(stream) }
		};
		
		match WebSocketRequest::read(wsstream.clone(), wsstream.clone()) {
			Ok(result) => { Ok(result) },
			Err(err) => {
				Err(IoError {
					kind: IoErrorKind::InvalidInput,
					desc: "Failed to read request",
					detail: Some(err.to_string()),
				})
			}
		}
    }
}