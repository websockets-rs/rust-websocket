//! Provides the default stream type for WebSocket connections.
use std::io::{
	self,
	Read,
	Write
};
use std::ops::Deref;
pub use std::net::{
	TcpStream,
	Shutdown,
};
pub use openssl::ssl::{
	SslStream,
	SslContext,
};

mod hyper;

pub trait AsTcpStream: Read + Write {
	fn as_tcp(&self) -> &TcpStream;
}

impl AsTcpStream for TcpStream {
	fn as_tcp(&self) -> &TcpStream {
		self
	}
}

impl AsTcpStream for SslStream<TcpStream> {
	fn as_tcp(&self) -> &TcpStream {
		self.get_ref()
	}
}

impl AsTcpStream for Box<AsTcpStream> {
	fn as_tcp(&self) -> &TcpStream {
		self.deref().as_tcp()
	}
}

pub trait TryUnsizedClone<C: ?Sized> {
	fn try_clone(&self) -> io::Result<Box<C>>;
}

impl TryUnsizedClone<AsTcpStream> for Box<AsTcpStream> {
	fn try_clone(&self) -> io::Result<Box<AsTcpStream>> {
		unimplemented!();
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

impl Stream for TcpStream {
	type R = Self;
	type W = Self;

	fn reader(&mut self) -> &mut Read {
		self
	}

	fn writer(&mut self) -> &mut Write {
		self
	}

	fn split(self) -> io::Result<(Self::R, Self::W)> {
		Ok((try!(self.try_clone()), self))
	}
}

impl Stream for SslStream<TcpStream> {
	type R = Self;
	type W = Self;

	fn reader(&mut self) -> &mut Read {
		self
	}

	fn writer(&mut self) -> &mut Write {
		self
	}

	fn split(self) -> io::Result<(Self::R, Self::W)> {
		Ok((try!(self.try_clone()), self))
	}
}

impl Stream for Box<AsTcpStream> {
	type R = Self;
	type W = Self;

	fn reader(&mut self) -> &mut Read {
		self
	}

	fn writer(&mut self) -> &mut Write {
		self
	}

	fn split(self) -> io::Result<(Self::R, Self::W)> {
		Ok((try!(self.try_clone()), self))
	}
}

#[derive(Clone)]
pub struct NoSslContext;

pub trait MaybeSslContext: Clone {}

impl MaybeSslContext for NoSslContext {}
impl MaybeSslContext for SslContext {}
