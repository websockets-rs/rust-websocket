#![stable]
//! Provides a default implementation of a stream which can be used with Rust-WebSocket.
//! 
//! A single type is provided, capable of carrying either a TcpStream or an SslStream<TcpStream>.
//! This is used with convenience methods such as ```WebSocketRequest::connect()```, ```WebSocketServer::bind())```
//! and ```WebSocketServer::bind_secure()```.

use std::io::net::tcp::TcpStream;
use std::io::IoResult;
use openssl::ssl::SslStream;

/// Represents a stream capable of carrying a WebSocket connection.
///
/// Can either be a normal TcpStream or an SSL-backed TcpStream.
/// This is the default stream used within Rust-WebSocket, however any Reader or Writer can be used
/// if desired, using the un-typed functions.
#[deriving(Clone)]
#[stable]
pub enum WebSocketStream {
	/// A normal (non-secure) stream
	Normal(TcpStream),
	/// A secure stream
	Secure(SslStream<TcpStream>)
}

impl Reader for WebSocketStream {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<uint> {
        match *self {
        	WebSocketStream::Normal(ref mut inner) => inner.read(buf),
			WebSocketStream::Secure(ref mut inner) => inner.read(buf)
		}
	}
}

impl Writer for WebSocketStream {
	fn write(&mut self, msg: &[u8]) -> IoResult<()> {
		match *self {
			WebSocketStream::Normal(ref mut inner) => inner.write(msg),
			WebSocketStream::Secure(ref mut inner) => inner.write(msg)
		}
	}
	fn flush(&mut self) -> IoResult<()> {
		match *self {
			WebSocketStream::Normal(ref mut inner) => inner.flush(),
			WebSocketStream::Secure(ref mut inner) => inner.flush(),
		}
	}
}