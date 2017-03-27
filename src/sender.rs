//! The default implementation of a WebSocket Sender.

use std::io::Write;
use std::io::Result as IoResult;
use result::WebSocketResult;
use ws::dataframe::DataFrame;
use stream::AsTcpStream;
use ws;
pub use stream::Shutdown;

pub struct Writer<W> {
    writer: W,
    sender: Sender,
}

impl<W> Writer<W>
    where W: Write,
{
	  /// Returns a reference to the underlying Writer.
	  pub fn get_ref(&self) -> &W {
		    &self.writer
	  }
	  /// Returns a mutable reference to the underlying Writer.
	  pub fn get_mut(&mut self) -> &mut W {
		    &mut self.writer
	  }
}

impl<S> Writer<S>
    where S: AsTcpStream + Write,
{
	  /// Closes the sender side of the connection, will cause all pending and future IO to
	  /// return immediately with an appropriate value.
	  pub fn shutdown(&self) -> IoResult<()> {
		    self.writer.as_tcp().shutdown(Shutdown::Write)
	  }

	  /// Shuts down both Sender and Receiver, will cause all pending and future IO to
	  /// return immediately with an appropriate value.
	  pub fn shutdown_all(&self) -> IoResult<()> {
		    self.writer.as_tcp().shutdown(Shutdown::Both)
	  }
}

/// A Sender that wraps a Writer and provides a default implementation using
/// DataFrames and Messages.
pub struct Sender {
	mask: bool,
}

impl Sender {
	/// Create a new WebSocketSender using the specified Writer.
	pub fn new(mask: bool) -> Sender {
		Sender {
			mask: mask,
		}
	}
}


impl ws::Sender for Sender {
	/// Sends a single data frame to the remote endpoint.
	  fn send_dataframe<D, W>(&mut self, writer: &mut W, dataframe: &D) -> WebSocketResult<()>
	      where D: DataFrame,
              W: Write,
    {
		dataframe.write_to(writer, self.mask)
	}
}
