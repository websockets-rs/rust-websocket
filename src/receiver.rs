//! The default implementation of a WebSocket Receiver.

use std::io::Read;
use std::io::Result as IoResult;

use hyper::buffer::BufReader;

use crate::dataframe::{DataFrame, Opcode};
use crate::message::OwnedMessage;
use crate::result::{WebSocketError, WebSocketResult};
pub use crate::stream::sync::Shutdown;
use crate::stream::sync::{AsTcpStream, Stream};
use crate::ws;
use crate::ws::receiver::Receiver as ReceiverTrait;
use crate::ws::receiver::{DataFrameIterator, MessageIterator};

const DEFAULT_MAX_DATAFRAME_SIZE : usize = 1024*1024*100;
const DEFAULT_MAX_MESSAGE_SIZE : usize = 1024*1024*200;
const MAX_DATAFRAMES_IN_ONE_MESSAGE: usize = 1024*1024;
const PER_DATAFRAME_OVERHEAD : usize = 64; // not actually measured, just to prevent filling memory with empty buffers

/// This reader bundles an existing stream with a parsing algorithm.
/// It is used by the client in its `.split()` function as the reading component.
pub struct Reader<R>
where
	R: Read,
{
	/// the stream to be read from
	pub stream: BufReader<R>,
	/// the parser to parse bytes into messages
	pub receiver: Receiver,
}

impl<R> Reader<R>
where
	R: Read,
{
	/// Reads a single data frame from the remote endpoint.
	pub fn recv_dataframe(&mut self) -> WebSocketResult<DataFrame> {
		self.receiver.recv_dataframe(&mut self.stream)
	}

	/// Returns an iterator over incoming data frames.
	pub fn incoming_dataframes(&mut self) -> DataFrameIterator<Receiver, BufReader<R>> {
		self.receiver.incoming_dataframes(&mut self.stream)
	}

	/// Reads a single message from this receiver.
	pub fn recv_message(&mut self) -> WebSocketResult<OwnedMessage> {
		self.receiver.recv_message(&mut self.stream)
	}

	/// An iterator over incoming messsages.
	/// This iterator will block until new messages arrive and will never halt.
	pub fn incoming_messages<'a>(&'a mut self) -> MessageIterator<'a, Receiver, BufReader<R>> {
		self.receiver.incoming_messages(&mut self.stream)
	}
}

impl<S> Reader<S>
where
	S: AsTcpStream + Stream + Read,
{
	/// Closes the receiver side of the connection, will cause all pending and future IO to
	/// return immediately with an appropriate value.
	pub fn shutdown(&self) -> IoResult<()> {
		self.stream.get_ref().as_tcp().shutdown(Shutdown::Read)
	}

	/// Shuts down both Sender and Receiver, will cause all pending and future IO to
	/// return immediately with an appropriate value.
	pub fn shutdown_all(&self) -> IoResult<()> {
		self.stream.get_ref().as_tcp().shutdown(Shutdown::Both)
	}
}

/// A Receiver that wraps a Reader and provides a default implementation using
/// DataFrames and Messages.
pub struct Receiver {
	buffer: Vec<DataFrame>,
	mask: bool,
	// u32s instead uf usizes to economize used memory by this struct
	max_dataframe_size: u32,
	max_message_size: u32,
}

impl Receiver {
	/// Create a new Receiver using the specified Reader.
	/// 
	/// Uses built-in limits for dataframe and message sizes. 
	pub fn new(mask: bool) -> Receiver {
		Receiver::new_with_limits(mask, DEFAULT_MAX_DATAFRAME_SIZE, DEFAULT_MAX_MESSAGE_SIZE)
	}

	/// Create a new Receiver using the specified Reader, with configurable limits
	/// 
	/// Sizes should not be larger than `u32::MAX`.
	/// 
	/// Note that `max_message_size` denotes message size where no new dataframes would be read,
	/// so actual maximum message size is larger.
	pub fn new_with_limits(mask: bool, max_dataframe_size: usize, max_message_size: usize) -> Receiver {
		let max_dataframe_size: u32 = max_dataframe_size.min(u32::MAX as usize) as u32;
		let max_message_size: u32 = max_message_size.min(u32::MAX as usize) as u32;
		Receiver {
			buffer: Vec::new(),
			mask,
			max_dataframe_size,
			max_message_size,
		}
	}
}

impl ws::Receiver for Receiver {
	type F = DataFrame;

	type M = OwnedMessage;

	/// Reads a single data frame from the remote endpoint.
	fn recv_dataframe<R>(&mut self, reader: &mut R) -> WebSocketResult<DataFrame>
	where
		R: Read,
	{
		DataFrame::read_dataframe_with_limit(reader, self.mask, self.max_dataframe_size as usize)
	}

	/// Returns the data frames that constitute one message.
	fn recv_message_dataframes<R>(&mut self, reader: &mut R) -> WebSocketResult<Vec<DataFrame>>
	where
		R: Read,
	{
		let mut current_message_length : usize = self.buffer.iter().map(|x|x.data.len()).sum();
		let mut finished = if self.buffer.is_empty() {
			let first = self.recv_dataframe(reader)?;

			if first.opcode == Opcode::Continuation {
				return Err(WebSocketError::ProtocolError(
					"Unexpected continuation data frame opcode",
				));
			}

			let finished = first.finished;
			current_message_length += first.data.len() + PER_DATAFRAME_OVERHEAD;
			self.buffer.push(first);
			finished
		} else {
			false
		};

		while !finished {
			let next = self.recv_dataframe(reader)?;
			finished = next.finished;

			match next.opcode as u8 {
				// Continuation opcode
				0 => {
					current_message_length += next.data.len() + PER_DATAFRAME_OVERHEAD;
					self.buffer.push(next)
				}
				// Control frame
				8..=15 => {
					return Ok(vec![next]);
				}
				// Others
				_ => {
					return Err(WebSocketError::ProtocolError(
						"Unexpected data frame opcode",
					));
				}
			}

			if !finished {
				if self.buffer.len() >= MAX_DATAFRAMES_IN_ONE_MESSAGE {
					return Err(WebSocketError::ProtocolError(
						"Exceeded count of data frames in one WebSocket message",
					));
				}
				if current_message_length >= self.max_message_size as usize {
					return Err(WebSocketError::ProtocolError(
						"Exceeded maximum WebSocket message size",
					));
				}
			}
		}

		Ok(::std::mem::replace(&mut self.buffer, Vec::new()))
	}
}
