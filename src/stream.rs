#![stable]
//! Provides the default stream type for WebSocket connections.

use std::old_io::IoResult;
use std::old_io::TcpStream;
use std::old_io::net::ip::SocketAddr;
use openssl::ssl::SslStream;

/// A useful stream type for carrying WebSocket connections.
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
	fn write_all(&mut self, msg: &[u8]) -> IoResult<()> {
		match *self {
			WebSocketStream::Tcp(ref mut inner) => inner.write_all(msg),
			WebSocketStream::Ssl(ref mut inner) => inner.write_all(msg),
		}
	}
}

impl WebSocketStream {
	/// See `TcpStream.peer_name()`.
	pub fn peer_name(&mut self) -> IoResult<SocketAddr> {
		match *self {
			WebSocketStream::Tcp(ref mut inner) => inner.peer_name(),
			WebSocketStream::Ssl(ref mut inner) => inner.get_mut().peer_name(),
		}
	}
	/// See `TcpStream.socket_name()`.
	pub fn socket_name(&mut self) -> IoResult<SocketAddr> {
		match *self {
			WebSocketStream::Tcp(ref mut inner) => inner.socket_name(),
			WebSocketStream::Ssl(ref mut inner) => inner.get_mut().socket_name(),
		}
	}
	/// See `TcpStream.set_nodelay()`.
	pub fn set_nodelay(&mut self, nodelay: bool) -> IoResult<()> {
		match *self {
			WebSocketStream::Tcp(ref mut inner) => inner.set_nodelay(nodelay),
			WebSocketStream::Ssl(ref mut inner) => inner.get_mut().set_nodelay(nodelay),
		}
	}
	/// See `TcpStream.set_keepalive()`.
	pub fn set_keepalive(&mut self, delay_in_seconds: Option<usize>) -> IoResult<()> {
		match *self {
			WebSocketStream::Tcp(ref mut inner) => inner.set_keepalive(delay_in_seconds),
			WebSocketStream::Ssl(ref mut inner) => inner.get_mut().set_keepalive(delay_in_seconds),
		}
	}
	/// See `TcpStream.close_read()`.
	pub fn close_read(&mut self) -> IoResult<()> {
		match *self {
			WebSocketStream::Tcp(ref mut inner) => inner.close_read(),
			WebSocketStream::Ssl(ref mut inner) => inner.get_mut().close_read(),
		}
	}
	/// See `TcpStream.close_write()`.
	pub fn close_write(&mut self) -> IoResult<()> {
		match *self {
			WebSocketStream::Tcp(ref mut inner) => inner.close_write(),
			WebSocketStream::Ssl(ref mut inner) => inner.get_mut().close_write(),
		}
	}
	/// See `TcpStream.set_timeout()`.
	pub fn set_timeout(&mut self, timeout_ms: Option<u64>) {
		match *self {
			WebSocketStream::Tcp(ref mut inner) => inner.set_timeout(timeout_ms),
			WebSocketStream::Ssl(ref mut inner) => inner.get_mut().set_timeout(timeout_ms),
		}
	}
	/// See `TcpStream.set_read_timeout()`.
	pub fn set_read_timeout(&mut self, timeout_ms: Option<u64>) {
		match *self {
			WebSocketStream::Tcp(ref mut inner) => inner.set_read_timeout(timeout_ms),
			WebSocketStream::Ssl(ref mut inner) => inner.get_mut().set_read_timeout(timeout_ms),
		}
	}
	/// See `TcpStream.set_write_timeout()`.
	pub fn set_write_timeout(&mut self, timeout_ms: Option<u64>) {
		match *self {
			WebSocketStream::Tcp(ref mut inner) => inner.set_write_timeout(timeout_ms),
			WebSocketStream::Ssl(ref mut inner) => inner.get_mut().set_write_timeout(timeout_ms),
		}
	}
}

impl Clone for WebSocketStream {
	fn clone(&self) -> WebSocketStream {
		match *self {
			WebSocketStream::Tcp(ref inner) => WebSocketStream::Tcp(inner.clone()),
			WebSocketStream::Ssl(ref inner) => WebSocketStream::Ssl(inner.clone()),
		}
	}
}
