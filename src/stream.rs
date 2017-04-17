//! Provides the default stream type for WebSocket connections.

use std::ops::Deref;
use std::fmt::Arguments;
use std::io::{self, Read, Write};
pub use std::net::TcpStream;
pub use std::net::Shutdown;
#[cfg(feature="ssl")]
pub use openssl::ssl::{SslStream, SslContext};

/// Represents a stream that can be read from, and written to.
/// This is an abstraction around readable and writable things to be able
/// to speak websockets over ssl, tcp, unix sockets, etc.
pub trait Stream: Read + Write {}

impl<S> Stream for S where S: Read + Write {}

/// a `Stream` that can also be used as a borrow to a `TcpStream`
/// this is useful when you want to set `TcpStream` options on a
/// `Stream` like `nonblocking`.
pub trait NetworkStream: Read + Write + AsTcpStream {}

impl<S> NetworkStream for S where S: Read + Write + AsTcpStream {}

/// some streams can be split up into separate reading and writing components
/// `TcpStream` is an example. This trait marks this ability so one can split
/// up the client into two parts.
///
/// Notice however that this is not possible to do with SSL.
pub trait Splittable {
	/// The reading component of this type
	type Reader: Read;
	/// The writing component of this type
	type Writer: Write;

	/// Split apart this type into a reading and writing component.
	fn split(self) -> io::Result<(Self::Reader, Self::Writer)>;
}

impl<R, W> Splittable for ReadWritePair<R, W>
	where R: Read,
	      W: Write
{
	type Reader = R;
	type Writer = W;

	fn split(self) -> io::Result<(R, W)> {
		Ok((self.0, self.1))
	}
}

impl Splittable for TcpStream {
	type Reader = TcpStream;
	type Writer = TcpStream;

	fn split(self) -> io::Result<(TcpStream, TcpStream)> {
		self.try_clone().map(|s| (s, self))
	}
}

/// The ability access a borrow to an underlying TcpStream,
/// so one can set options on the stream such as `nonblocking`.
pub trait AsTcpStream {
	/// Get a borrow of the TcpStream
	fn as_tcp(&self) -> &TcpStream;
}

impl AsTcpStream for TcpStream {
	fn as_tcp(&self) -> &TcpStream {
		self
	}
}

#[cfg(feature="ssl")]
impl AsTcpStream for SslStream<TcpStream> {
	fn as_tcp(&self) -> &TcpStream {
		self.get_ref()
	}
}

impl<T> AsTcpStream for Box<T>
    where T: AsTcpStream
{
	fn as_tcp(&self) -> &TcpStream {
		self.deref().as_tcp()
	}
}

/// If you would like to combine an input stream and an output stream into a single
/// stream to talk websockets over then this is the struct for you!
///
/// This is useful if you want to use different mediums for different directions.
pub struct ReadWritePair<R, W>(pub R, pub W)
	where R: Read,
	      W: Write;

impl<R, W> Read for ReadWritePair<R, W>
	where R: Read,
	      W: Write
{
	#[inline(always)]
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		self.0.read(buf)
	}
	#[inline(always)]
	fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
		self.0.read_to_end(buf)
	}
	#[inline(always)]
	fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
		self.0.read_to_string(buf)
	}
	#[inline(always)]
	fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
		self.0.read_exact(buf)
	}
}

impl<R, W> Write for ReadWritePair<R, W>
	where R: Read,
	      W: Write
{
	#[inline(always)]
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		self.1.write(buf)
	}
	#[inline(always)]
	fn flush(&mut self) -> io::Result<()> {
		self.1.flush()
	}
	#[inline(always)]
	fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
		self.1.write_all(buf)
	}
	#[inline(always)]
	fn write_fmt(&mut self, fmt: Arguments) -> io::Result<()> {
		self.1.write_fmt(fmt)
	}
}
