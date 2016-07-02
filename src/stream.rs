//! Provides the default stream type for WebSocket connections.
extern crate net2;

use std::io::{self, Read, Write};
use self::net2::TcpStreamExt;
use openssl::ssl::SslStream;

pub use std::net::{SocketAddr, Shutdown, TcpStream};

/// Represents a stream that can be read from, written to, and split into two.
/// This is an abstraction around readable and writable things to be able
/// to speak websockets over ssl, tcp, unix sockets, etc.
pub trait Stream<R, W>
where R: Read,
      W: Write,
{
    /// Get a mutable borrow to the reading component of this stream
	fn reader(&mut self) -> &mut R;

    /// Get a mutable borrow to the writing component of this stream
	fn writer(&mut self) -> &mut W;

    /// Split this stream into readable and writable components.
    /// The motivation behind this is to be able to read on one thread
    /// and send messages on another.
	fn split(self) -> Result<(R, W), io::Error>;
}

impl<R, W> Stream<R, W> for (R, W)
where R: Read,
      W: Write,
{
	fn reader(&mut self) -> &mut R {
		&mut self.0
	}

	fn writer(&mut self) -> &mut W {
		&mut self.1
	}

	fn split(self) -> Result<(R, W), io::Error> {
		Ok(self)
	}
}

impl Stream<TcpStream, TcpStream> for TcpStream {
	fn reader(&mut self) -> &mut TcpStream {
		self
	}

	fn writer(&mut self) -> &mut TcpStream {
		self
	}

	fn split(self) -> Result<(TcpStream, TcpStream), io::Error> {
		Ok((try!(self.try_clone()), self))
	}
}

impl Stream<SslStream<TcpStream>, SslStream<TcpStream>> for SslStream<TcpStream> {
	fn reader(&mut self) -> &mut SslStream<TcpStream> {
		self
	}

	fn writer(&mut self) -> &mut SslStream<TcpStream> {
		self
	}

	fn split(self) -> Result<(SslStream<TcpStream>, SslStream<TcpStream>), io::Error> {
		Ok((try!(self.try_clone()), self))
	}
}

/// A useful stream type for carrying WebSocket connections.
pub enum WebSocketStream {
	/// A TCP stream.
	Tcp(TcpStream),
	/// An SSL-backed TCP Stream
	Ssl(SslStream<TcpStream>)
}

impl Read for WebSocketStream {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		match *self {
		WebSocketStream::Tcp(ref mut inner) => inner.read(buf),
			WebSocketStream::Ssl(ref mut inner) => inner.read(buf),
		}
	}
}

impl Write for WebSocketStream {
	fn write(&mut self, msg: &[u8]) -> io::Result<usize> {
		match *self {
			WebSocketStream::Tcp(ref mut inner) => inner.write(msg),
			WebSocketStream::Ssl(ref mut inner) => inner.write(msg),
		}
	}

	fn flush(&mut self) -> io::Result<()> {
		match *self {
			WebSocketStream::Tcp(ref mut inner) => inner.flush(),
			WebSocketStream::Ssl(ref mut inner) => inner.flush(),
		}
	}
}

impl WebSocketStream {
	/// See `TcpStream.peer_addr()`.
	pub fn peer_addr(&self) -> io::Result<SocketAddr> {
		match *self {
			WebSocketStream::Tcp(ref inner) => inner.peer_addr(),
			WebSocketStream::Ssl(ref inner) => inner.get_ref().peer_addr(),
		}
	}
	/// See `TcpStream.local_addr()`.
	pub fn local_addr(&self) -> io::Result<SocketAddr> {
		match *self {
			WebSocketStream::Tcp(ref inner) => inner.local_addr(),
			WebSocketStream::Ssl(ref inner) => inner.get_ref().local_addr(),
		}
	}
	/// See `TcpStream.set_nodelay()`.
	pub fn set_nodelay(&mut self, nodelay: bool) -> io::Result<()> {
		match *self {
			WebSocketStream::Tcp(ref mut inner) => TcpStreamExt::set_nodelay(inner, nodelay),
			WebSocketStream::Ssl(ref mut inner) => TcpStreamExt::set_nodelay(inner.get_mut(), nodelay),
		}
	}
	/// See `TcpStream.set_keepalive()`.
	pub fn set_keepalive(&mut self, delay_in_ms: Option<u32>) -> io::Result<()> {
		match *self {
			WebSocketStream::Tcp(ref mut inner) => TcpStreamExt::set_keepalive_ms(inner, delay_in_ms),
			WebSocketStream::Ssl(ref mut inner) => TcpStreamExt::set_keepalive_ms(inner.get_mut(), delay_in_ms),
		}
	}
	/// See `TcpStream.shutdown()`.
	pub fn shutdown(&mut self, shutdown: Shutdown) -> io::Result<()> {
		match *self {
			WebSocketStream::Tcp(ref mut inner) => inner.shutdown(shutdown),
			WebSocketStream::Ssl(ref mut inner) => inner.get_mut().shutdown(shutdown),
		}
	}
	/// See `TcpStream.try_clone()`.
	pub fn try_clone(&self) -> io::Result<WebSocketStream> {
		Ok(match *self {
			WebSocketStream::Tcp(ref inner) => WebSocketStream::Tcp(try!(inner.try_clone())),
			WebSocketStream::Ssl(ref inner) => WebSocketStream::Ssl(try!(inner.try_clone())),
		})
	}

    /// Changes whether the stream is in nonblocking mode.
    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        match *self {
			WebSocketStream::Tcp(ref inner) => inner.set_nonblocking(nonblocking),
			WebSocketStream::Ssl(ref inner) => inner.get_ref().set_nonblocking(nonblocking),
        }
    }
}
