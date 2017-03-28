//! The default implementation of a WebSocket Receiver.

use std::io::Read;
use std::io::Result as IoResult;

use dataframe::{
    DataFrame,
    Opcode
};
use result::{
    WebSocketResult,
    WebSocketError
};
use ws;
use ws::dataframe::DataFrame as DataFrameable;
use ws::receiver::Receiver as ReceiverTrait;
use ws::receiver::{
    MessageIterator,
    DataFrameIterator,
};
use stream::{
    AsTcpStream,
    Stream,
};
pub use stream::Shutdown;

// TODO: buffer the readers
pub struct Reader<R>
    where R: Read
{
    pub reader: R,
    pub receiver: Receiver,
}

impl<R> Reader<R>
    where R: Read,
{
	  /// Reads a single data frame from the remote endpoint.
	  pub fn recv_dataframe(&mut self) -> WebSocketResult<DataFrame> {
		    self.receiver.recv_dataframe(&mut self.reader)
	  }

	  /// Returns an iterator over incoming data frames.
	  pub fn incoming_dataframes<'a>(&'a mut self) -> DataFrameIterator<'a, Receiver, R> {
		    self.receiver.incoming_dataframes(&mut self.reader)
	  }

	  /// Reads a single message from this receiver.
	  pub fn recv_message<'m, M, I>(&mut self) -> WebSocketResult<M>
	      where M: ws::Message<'m, DataFrame, DataFrameIterator = I>,
              I: Iterator<Item = DataFrame>
    {
		    self.receiver.recv_message(&mut self.reader)
	  }

	  pub fn incoming_messages<'a, M, D>(&'a mut self) -> MessageIterator<'a, Receiver, D, M, R>
	      where M: ws::Message<'a, D>,
              D: DataFrameable
    {
		    self.receiver.incoming_messages(&mut self.reader)
	  }
}

impl<S> Reader<S>
    where S: AsTcpStream + Stream + Read,
{
	  /// Closes the receiver side of the connection, will cause all pending and future IO to
	  /// return immediately with an appropriate value.
	  pub fn shutdown(&self) -> IoResult<()> {
		    self.reader.as_tcp().shutdown(Shutdown::Read)
	  }

	  /// Shuts down both Sender and Receiver, will cause all pending and future IO to
	  /// return immediately with an appropriate value.
	  pub fn shutdown_all(&self) -> IoResult<()> {
		    self.reader.as_tcp().shutdown(Shutdown::Both)
	  }
}

/// A Receiver that wraps a Reader and provides a default implementation using
/// DataFrames and Messages.
pub struct Receiver {
	buffer: Vec<DataFrame>,
	mask: bool,
}

impl Receiver {
	/// Create a new Receiver using the specified Reader.
	pub fn new(mask: bool) -> Receiver {
		Receiver {
			buffer: Vec::new(),
			mask: mask,
		}
	}
}


impl ws::Receiver for Receiver {
    type F = DataFrame;

	/// Reads a single data frame from the remote endpoint.
	  fn recv_dataframe<R>(&mut self, reader: &mut R) -> WebSocketResult<DataFrame>
        where R: Read,
    {
		DataFrame::read_dataframe(reader, self.mask)
	}

	/// Returns the data frames that constitute one message.
	  fn recv_message_dataframes<R>(&mut self, reader: &mut R) -> WebSocketResult<Vec<DataFrame>>
        where R: Read,
    {
		let mut finished = if self.buffer.is_empty() {
			let first = try!(self.recv_dataframe(reader));

			if first.opcode == Opcode::Continuation {
				return Err(WebSocketError::ProtocolError(
					"Unexpected continuation data frame opcode"
				));
			}

			let finished = first.finished;
			self.buffer.push(first);
			finished
		}
		else {
			false
		};

		while !finished {
			let next = try!(self.recv_dataframe(reader));
			finished = next.finished;

			match next.opcode as u8 {
				// Continuation opcode
				0 => self.buffer.push(next),
				// Control frame
				8...15 => {
					return Ok(vec![next]);
				}
				// Others
				_ => return Err(WebSocketError::ProtocolError(
					"Unexpected data frame opcode"
				)),
			}
		}

		let buffer = self.buffer.clone();
		self.buffer.clear();

		Ok(buffer)
	}
}
