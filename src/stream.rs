//! Provides the default stream type for WebSocket connections.
extern crate net2;

use std::io::{self, Read, Write};
use self::net2::TcpStreamExt;
#[cfg(feature="ssl")]
use openssl::ssl::SslStream;

pub use std::net::{SocketAddr, Shutdown, TcpStream};

/// A useful stream type for carrying WebSocket connections.
pub enum WebSocketStream {
	/// A TCP stream.
	Tcp(TcpStream),
	/// An SSL-backed TCP Stream
	#[cfg(feature="ssl")]
	Ssl(SslStream<TcpStream>),
	/// An opaque pair of arbitrary Read and Write to server as backend
	Custom((Box<Read+Send>, Box<Write+Send>)),
}

impl Read for WebSocketStream {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		match *self {
		WebSocketStream::Tcp(ref mut inner) => inner.read(buf),
			#[cfg(feature="ssl")]
			WebSocketStream::Ssl(ref mut inner) => inner.read(buf),
			WebSocketStream::Custom((ref mut r, _)) => r.read(buf),
		}
	}
}

impl Write for WebSocketStream {
	fn write(&mut self, msg: &[u8]) -> io::Result<usize> {
		match *self {
			WebSocketStream::Tcp(ref mut inner) => inner.write(msg),
			#[cfg(feature="ssl")]
			WebSocketStream::Ssl(ref mut inner) => inner.write(msg),
			WebSocketStream::Custom((_,ref mut w)) => w.write(msg),
		}
	}

	fn flush(&mut self) -> io::Result<()> {
		match *self {
			WebSocketStream::Tcp(ref mut inner) => inner.flush(),
			#[cfg(feature="ssl")]
			WebSocketStream::Ssl(ref mut inner) => inner.flush(),
			WebSocketStream::Custom((_,ref mut w)) => w.flush(),
		}
	}
}

impl WebSocketStream {
	/// See `TcpStream.peer_addr()`.
	pub fn peer_addr(&self) -> io::Result<SocketAddr> {
		match *self {
			WebSocketStream::Tcp(ref inner) => inner.peer_addr(),
			#[cfg(feature="ssl")]
			WebSocketStream::Ssl(ref inner) => inner.get_ref().peer_addr(),
			WebSocketStream::Custom(_) => 
				Err(io::Error::new(io::ErrorKind::AddrNotAvailable, 
					"no address for custom-backed websocket")),
		}
	}
	/// See `TcpStream.local_addr()`.
	pub fn local_addr(&self) -> io::Result<SocketAddr> {
		match *self {
			WebSocketStream::Tcp(ref inner) => inner.local_addr(),
			#[cfg(feature="ssl")]
			WebSocketStream::Ssl(ref inner) => inner.get_ref().local_addr(),
			WebSocketStream::Custom(_) => 
				Err(io::Error::new(io::ErrorKind::AddrNotAvailable, 
					"no address for custom-backed websocket")),
		}
	}
	/// See `TcpStream.set_nodelay()`.
	pub fn set_nodelay(&mut self, nodelay: bool) -> io::Result<()> {
		match *self {
			WebSocketStream::Tcp(ref mut inner) => TcpStreamExt::set_nodelay(inner, nodelay),
			#[cfg(feature="ssl")]
			WebSocketStream::Ssl(ref mut inner) => TcpStreamExt::set_nodelay(inner.get_mut(), nodelay),
			WebSocketStream::Custom(_) => Err(io::Error::new(io::ErrorKind::Other, 
					"no nodelay for custom-backed websocket")),
		}
	}
	/// See `TcpStream.set_keepalive()`.
	pub fn set_keepalive(&mut self, delay_in_ms: Option<u32>) -> io::Result<()> {
		match *self {
			WebSocketStream::Tcp(ref mut inner) => TcpStreamExt::set_keepalive_ms(inner, delay_in_ms),
			#[cfg(feature="ssl")]
			WebSocketStream::Ssl(ref mut inner) => TcpStreamExt::set_keepalive_ms(inner.get_mut(), delay_in_ms),
			WebSocketStream::Custom(_) => Err(io::Error::new(io::ErrorKind::Other, 
					"no keepalive for custom-backed websocket")),
		}
	}
	/// See `TcpStream.shutdown()`.
	pub fn shutdown(&mut self, shutdown: Shutdown) -> io::Result<()> {
		match *self {
			WebSocketStream::Tcp(ref mut inner) => inner.shutdown(shutdown),
			#[cfg(feature="ssl")]
			WebSocketStream::Ssl(ref mut inner) => inner.get_mut().shutdown(shutdown),
			WebSocketStream::Custom(_) => Err(io::Error::new(io::ErrorKind::Other, 
					"can't shutdown custom-backed websocket")),
		}
	}
	/// See `TcpStream.try_clone()`.
	pub fn try_clone(&self) -> io::Result<WebSocketStream> {
		Ok(match *self {
			WebSocketStream::Tcp(ref inner) => WebSocketStream::Tcp(try!(inner.try_clone())),
			#[cfg(feature="ssl")]
			WebSocketStream::Ssl(ref inner) => WebSocketStream::Ssl(try!(inner.try_clone())),
			WebSocketStream::Custom(_) => Err(io::Error::new(io::ErrorKind::Other, 
					"can't clone custom-backed websocket"))?,
		})
	}

    /// Changes whether the stream is in nonblocking mode.
    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        match *self {
			WebSocketStream::Tcp(ref inner) => inner.set_nonblocking(nonblocking),
			#[cfg(feature="ssl")]
			WebSocketStream::Ssl(ref inner) => inner.get_ref().set_nonblocking(nonblocking),
			WebSocketStream::Custom(_) => Err(io::Error::new(io::ErrorKind::Other, 
					"no nonblocking for custom-backed websocket")),
        }
    }
}
