//! Provides an implementation of a WebSocket server
use std::net::{
	  SocketAddr,
	  ToSocketAddrs,
	  TcpListener,
	  TcpStream,
	  Shutdown,
};
use std::io::{
	  self,
	  Read,
	  Write,
};
use std::borrow::Cow;
use std::ops::Deref;
use std::convert::Into;
use openssl::ssl::{
	  SslContext,
	  SslMethod,
	  SslStream,
    SslAcceptor,
};
use stream::{
	  Stream,
};
use self::upgrade::{
	  WsUpgrade,
	  IntoWs,
};
pub use self::upgrade::{
	  Request,
	  HyperIntoWsError,
};

pub mod upgrade;

pub struct InvalidConnection<S>
    where S: Stream,
{
	  pub stream: Option<S>,
	  pub parsed: Option<Request>,
	  pub error: HyperIntoWsError,
}

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
impl OptionalSslAcceptor for SslAcceptor {}

/// Represents a WebSocket server which can work with either normal (non-secure) connections, or secure WebSocket connections.
///
/// This is a convenient way to implement WebSocket servers, however it is possible to use any sendable Reader and Writer to obtain
/// a WebSocketClient, so if needed, an alternative server implementation can be used.
///#Non-secure Servers
///
/// ```no_run
///extern crate websocket;
///# fn main() {
///use std::thread;
///use websocket::{Server, Message};
///
///let server = Server::bind("127.0.0.1:1234").unwrap();
///
///for connection in server {
///    // Spawn a new thread for each connection.
///    thread::spawn(move || {
///		   let request = connection.unwrap().read_request().unwrap(); // Get the request
///		   let response = request.accept(); // Form a response
///		   let mut client = response.send().unwrap(); // Send the response
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
///#Secure Servers
/// ```no_run
///extern crate websocket;
///extern crate openssl;
///# fn main() {
///use std::thread;
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
///for connection in server {
///    // Spawn a new thread for each connection.
///    thread::spawn(move || {
///		   let request = connection.unwrap().read_request().unwrap(); // Get the request
///		   let response = request.accept(); // Form a response
///		   let mut client = response.send().unwrap(); // Send the response
///
///		   let message = Message::text("Hello, client!");
///		   let _ = client.send_message(&message);
///
///		   // ...
///    });
///}
/// # }
/// ```
pub struct Server<S>
    where S: OptionalSslAcceptor,
{
	  pub listener: TcpListener,
	  ssl_acceptor: S,
}

impl<S> Server<S>
    where S: OptionalSslAcceptor,
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
}

impl Server<SslAcceptor> {
	  /// Bind this Server to this socket, utilising the given SslContext
	  pub fn bind_secure<A>(addr: A, acceptor: Option<SslAcceptor>) -> io::Result<Self>
	      where A: ToSocketAddrs,
	  {
		    Ok(Server {
			      listener: try!(TcpListener::bind(&addr)),
			      ssl_acceptor: match acceptor {
                Some(acc) => acc,
                None => {
                    unimplemented!();
                }
            },
		    })
	  }

	  /// Wait for and accept an incoming WebSocket connection, returning a WebSocketRequest
	  pub fn accept(&mut self) -> AcceptResult<SslStream<TcpStream>> {
		    let stream = match self.listener.accept() {
			      Ok(s) => s.0,
			      Err(e) => return Err(InvalidConnection {
				        stream: None,
				        parsed: None,
				        error: e.into(),
			      }),
		    };

		    let stream = match self.ssl_acceptor.accept(stream) {
			      Ok(s) => s,
			      Err(err) => return Err(InvalidConnection {
				        stream: None,
				        parsed: None,
				        error: io::Error::new(io::ErrorKind::Other, err).into(),
			      }),
		    };

		    match stream.into_ws() {
			      Ok(u) => Ok(u),
			      Err((s, r, e)) => Err(InvalidConnection {
				        stream: Some(s),
				        parsed: r,
				        error: e.into(),
			      }),
		    }
	  }

    /// Changes whether the Server is in nonblocking mode.
    ///
    /// If it is in nonblocking mode, accept() will return an error instead of blocking when there
    /// are no incoming connections.
    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        self.listener.set_nonblocking(nonblocking)
    }
}

impl Iterator for Server<SslAcceptor> {
	  type Item = WsUpgrade<SslStream<TcpStream>>;

	  fn next(&mut self) -> Option<<Self as Iterator>::Item> {
		    self.accept().ok()
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
			      Err(e) => return Err(InvalidConnection {
				        stream: None,
				        parsed: None,
				        error: e.into(),
			      }),
		    };

		    match stream.into_ws() {
			      Ok(u) => Ok(u),
			      Err((s, r, e)) => Err(InvalidConnection {
				        stream: Some(s),
				        parsed: r,
				        error: e.into(),
			      }),
		    }
	  }
}

impl Iterator for Server<NoSslAcceptor> {
	  type Item = WsUpgrade<TcpStream>;

	  fn next(&mut self) -> Option<<Self as Iterator>::Item> {
		    self.accept().ok()
	  }
}

