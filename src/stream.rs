//! Provides the default stream type for WebSocket connections.
// TODO: add mio support & tokio
// extern crate mio;

use std::io::{
	  self,
	  Read,
	  Write
};
pub use std::net::TcpStream;
pub use std::net::Shutdown;
pub use openssl::ssl::{
	  SslStream,
	  SslContext,
};

pub trait Splittable {
	  type Reader: Read;
    type Writer: Write;

	  fn split(self) -> io::Result<(Self::Reader, Self::Writer)>;
}

/// Represents a stream that can be read from, and written to.
/// This is an abstraction around readable and writable things to be able
/// to speak websockets over ssl, tcp, unix sockets, etc.
pub trait Stream {
	  type Reader: Read;
    type Writer: Write;

	  /// Get a mutable borrow to the reading component of this stream
	  fn reader(&mut self) -> &mut Self::Reader;

	  /// Get a mutable borrow to the writing component of this stream
	  fn writer(&mut self) -> &mut Self::Writer;
}

pub struct ReadWritePair<R, W>(pub R, pub W)
    where R: Read,
	        W: Write;

impl<R, W> Splittable for ReadWritePair<R, W>
    where R: Read,
	        W: Write,
{
	  type Reader = R;
    type Writer = W;

	  fn split(self) -> io::Result<(R, W)> {
		    Ok((self.0, self.1))
	  }
}

impl<R, W> Stream for ReadWritePair<R, W>
    where R: Read,
	        W: Write,
{
    type Reader = R;
    type Writer = W;

	  #[inline]
	  fn reader(&mut self) -> &mut R {
		    &mut self.0
	  }

	  #[inline]
	  fn writer(&mut self) -> &mut W {
		    &mut self.1
	  }
}

impl Splittable for TcpStream {
	  type Reader = TcpStream;
    type Writer = TcpStream;

	  fn split(self) -> io::Result<(TcpStream, TcpStream)> {
		    self.try_clone().map(|s| (s, self))
	  }
}

impl<S> Stream for S
    where S: Read + Write,
{
    type Reader = Self;
    type Writer = Self;

	  #[inline]
	  fn reader(&mut self) -> &mut S {
		    self
	  }

	  #[inline]
	  fn writer(&mut self) -> &mut S {
		    self
	  }
}

pub trait AsTcpStream {
    fn as_tcp(&self) -> &TcpStream;
}

impl AsTcpStream for TcpStream {
    fn as_tcp(&self) -> &TcpStream {
        &self
    }
}

impl AsTcpStream for SslStream<TcpStream> {
    fn as_tcp(&self) -> &TcpStream {
        self.get_ref()
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
