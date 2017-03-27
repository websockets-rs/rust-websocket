//! The default implementation of a WebSocket Sender.

use std::io::Write;
use std::io::Result as IoResult;
use result::WebSocketResult;
use ws::dataframe::DataFrame;
use stream::AsTcpStream;
use ws;
use ws::sender::Sender as SenderTrait;
pub use stream::Shutdown;

pub struct Writer<W> {
    pub writer: W,
    pub sender: Sender,
}

impl<W> Writer<W>
    where W: Write,
{
	  /// Sends a single data frame to the remote endpoint.
	  fn send_dataframe<D>(&mut self, dataframe: &D) -> WebSocketResult<()>
	      where D: DataFrame,
              W: Write,
    {
        self.sender.send_dataframe(&mut self.writer, dataframe)
    }

	  /// Sends a single message to the remote endpoint.
	  pub fn send_message<'m, M, D>(&mut self, message: &'m M) -> WebSocketResult<()>
	      where M: ws::Message<'m, D>,
              D: DataFrame
    {
		    self.sender.send_message(&mut self.writer, message)
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
