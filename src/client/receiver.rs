//! The default implementation of a WebSocket Receiver.

use std::io::Read;
use std::io::Result as IoResult;
use hyper::buffer::BufReader;

use dataframe::{DataFrame, Opcode};
use result::{WebSocketResult, WebSocketError};
use stream::WebSocketStream;
use stream::Shutdown;
use ws;

/// A Receiver that wraps a Reader and provides a default implementation using
/// DataFrames and Messages.
pub struct Receiver<R> {
	inner: BufReader<R>,
	buffer: Vec<DataFrame>
}

impl<R> Receiver<R> {
	/// Create a new Receiver using the specified Reader.
	pub fn new(reader: BufReader<R>) -> Receiver<R> {
		Receiver {
			inner: reader,
			buffer: Vec::new()
		}
	}
	/// Returns a reference to the underlying Reader.
	pub fn get_ref(&self) -> &BufReader<R> {
		&self.inner
	}
	/// Returns a mutable reference to the underlying Reader.
	pub fn get_mut(&mut self) -> &mut BufReader<R> {
		&mut self.inner
	}
}

impl Receiver<WebSocketStream> {
    /// Closes the receiver side of the connection, will cause all pending and future IO to
    /// return immediately with an appropriate value.
    pub fn shutdown(&mut self) -> IoResult<()> {
        self.inner.get_mut().shutdown(Shutdown::Read)
    }

    /// Shuts down both Sender and Receiver, will cause all pending and future IO to
    /// return immediately with an appropriate value.
    pub fn shutdown_all(&mut self) -> IoResult<()> {
        self.inner.get_mut().shutdown(Shutdown::Both)
    }
}

impl<R: Read> ws::Receiver<DataFrame> for Receiver<R> {
	/// Reads a single data frame from the remote endpoint.
	fn recv_dataframe(&mut self) -> WebSocketResult<DataFrame> {
		DataFrame::read_dataframe(&mut self.inner, false)
	}
	/// Returns the data frames that constitute one message.
	fn recv_message_dataframes(&mut self) -> WebSocketResult<Vec<DataFrame>> {
		let mut finished = if self.buffer.is_empty() {
			let first = try!(self.recv_dataframe());

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
			let next = try!(self.recv_dataframe());
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
