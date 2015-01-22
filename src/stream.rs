//! Provides the default stream type for WebSocket connections.

use std::io::net::tcp::TcpStream;
use std::io::IoResult;
use openssl::ssl::SslStream;

/// A useful stream type for carrying WebSocket connections.
#[derive(Clone)]
pub enum WebSocketStream {
	/// A TCP stream.
	Tcp(TcpStream),
	/// An SSL-backed TCP Stream
	Ssl(SslStream<TcpStream>)
}

impl Reader for WebSocketStream {
	fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
		match *self {
        	WebSocketStream::Tcp(ref mut inner) => inner.read(buf),
			WebSocketStream::Ssl(ref mut inner) => inner.read(buf),
		}
	}
}

impl Writer for WebSocketStream {
	fn write(&mut self, msg: &[u8]) -> IoResult<()> {
		match *self {
			WebSocketStream::Tcp(ref mut inner) => inner.write(msg),
			WebSocketStream::Ssl(ref mut inner) => inner.write(msg),
		}
	}
}