//! The default implementation of a WebSocket Receiver.

use dataframe::DataFrame;
use message::Message;
use result::{WebSocketResult, WebSocketError};
use ws::util::dataframe::read_dataframe;
use ws;

/// A Receiver that wraps a Reader and provides a default implementation using
/// DataFrames and Messages.
pub struct Receiver<R> {
	inner: R,
	buffer: Vec<DataFrame>
}

impl<R> Receiver<R> {
	/// Create a new Receiver using the specified Reader.
	pub fn new(reader: R) -> Receiver<R> {
		Receiver {
			inner: reader,
			buffer: Vec::new()
		}
	}
	/// Returns a reference to the underlying Reader.
	pub fn get_ref(&self) -> &R {
		&self.inner
	}
	/// Returns a mutable reference to the underlying Reader.
	pub fn get_mut(&mut self) -> &mut R {
		&mut self.inner
	}
}

impl<R: Reader> ws::Receiver<DataFrame> for Receiver<R> {
	type Message = Message;
	
	fn recv_dataframe(&mut self) -> WebSocketResult<DataFrame> {
		match self.buffer.pop() {
			Some(dataframe) => Ok(dataframe),
			None => read_dataframe(&mut self.inner, true),
		}
	}
	fn recv_message(&mut self) -> WebSocketResult<Message> {
		let first = try!(self.recv_dataframe());
		
		let mut finished = first.finished;
		let mut buffer = Vec::new();
		let mut frames = Vec::new();
		
		frames.push(first);
		
		while !finished {
			let next = try!(self.recv_dataframe());
			finished = next.finished;
			
			match next.opcode as u8 {
				// Continuation opcode
				0 => frames.push(next),
				// Control frame
				8...15 => buffer.push(next),
				// Others
				_ => return Err(WebSocketError::ProtocolError(
					"Unexpected data frame opcode".to_string()
				)),
			}
		}
		
		self.buffer.push_all(&buffer[]);

		ws::Message::from_iter(frames.into_iter())
	}
}