//! Provides the default stream type for WebSocket connections.
use std::ops::Deref;
use std::io::{
	self,
	Read,
	Write
};
pub use std::net::{
	TcpStream,
	Shutdown,
};
pub use openssl::ssl::{
	SslStream,
	SslContext,
};

pub trait AsTcpStream: Read + Write {
	fn as_tcp(&self) -> &TcpStream;

	fn duplicate(&self) -> io::Result<Self>
	where Self: Sized;

	fn box_duplicate(&self) -> io::Result<Box<AsTcpStream>>;
}

impl AsTcpStream for TcpStream {
	fn as_tcp(&self) -> &TcpStream {
		self
	}

	fn duplicate(&self) -> io::Result<Self> {
		self.try_clone()
	}

	fn box_duplicate(&self) -> io::Result<Box<AsTcpStream>> {
		Ok(Box::new(try!(self.duplicate())))
	}
}

impl AsTcpStream for SslStream<TcpStream> {
	fn as_tcp(&self) -> &TcpStream {
		self.get_ref()
	}

	fn duplicate(&self) -> io::Result<Self> {
		self.try_clone()
	}

	fn box_duplicate(&self) -> io::Result<Box<AsTcpStream>> {
		Ok(Box::new(try!(self.duplicate())))
	}
}

impl AsTcpStream for Box<AsTcpStream> {
	fn as_tcp(&self) -> &TcpStream {
		self.deref().as_tcp()
	}

	fn duplicate(&self) -> io::Result<Self> {
		self.deref().box_duplicate()
	}

	fn box_duplicate(&self) -> io::Result<Box<AsTcpStream>> {
		self.duplicate()
	}
}

/// Represents a stream that can be read from, written to, and split into two.
/// This is an abstraction around readable and writable things to be able
/// to speak websockets over ssl, tcp, unix sockets, etc.
pub trait Stream
{
	/// The reading component of the stream
	type R: Read;
	/// The writing component of the stream
	type W: Write;

	/// Get a mutable borrow to the reading component of this stream
	fn reader(&mut self) -> &mut Read;

	/// Get a mutable borrow to the writing component of this stream
	fn writer(&mut self) -> &mut Write;

	/// Split this stream into readable and writable components.
	/// The motivation behind this is to be able to read on one thread
	/// and send messages on another.
	fn split(self) -> io::Result<(Self::R, Self::W)>;
}

impl<R, W> Stream for (R, W)
where R: Read,
	  W: Write,
{
	type R = R;
	type W = W;

	fn reader(&mut self) -> &mut Read {
		&mut self.0
	}

	fn writer(&mut self) -> &mut Write {
		&mut self.1
	}

	fn split(self) -> io::Result<(Self::R, Self::W)> {
		Ok(self)
	}
}

impl<S> Stream for S
where S: AsTcpStream,
{
	type R = Self;
	type W = Self;

	fn reader(&mut self) -> &mut Read {
		self
	}

	fn writer(&mut self) -> &mut Write {
		self
	}

	fn split(self) -> io::Result<(Self::R, Self::W)> {
		Ok((try!(self.duplicate()), self))
	}
}

/// Marker struct for having no SSL context in a struct.
#[derive(Clone)]
pub struct NoSslContext;
/// Trait that is implemented over NoSslContext and SslContext that
/// serves as a generic bound to make a struct with.
/// Used in the Server to specify impls based on wether the server
/// is running over SSL or not.
pub trait MaybeSslContext: Clone {}
impl MaybeSslContext for NoSslContext {}
impl MaybeSslContext for SslContext {}
