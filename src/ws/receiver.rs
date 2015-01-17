//! Provides a trait for receiving data frames and messages.
//!
//! Also provides iterators over data frames and messages.
//! See the `ws` module documentation for more information.

use ws::Message;
use result::WebSocketResult;

/// A trait for receiving data frames and messages.
pub trait Receiver<D>: Sized {
	/// The type of message received by this Receiver.
	type Message: Message<D>;
	/// Reads a single data frame from this receiver.
	fn recv_dataframe(&mut self) -> WebSocketResult<D>;
	/// Returns an iterator over incoming data frames.
	fn incoming_dataframes<'a>(&'a mut self) -> DataFrameIterator<'a, Self, D> {
		DataFrameIterator {
			inner: self
		}
	}
	/// Reads a single message from this receiver.
	fn recv_message(&mut self) -> WebSocketResult<Self::Message>;
	/// Returns an iterator over incoming messages.
	fn incoming_messages<'a>(&'a mut self) -> MessageIterator<'a, Self, D> {
		MessageIterator {
			inner: self
		}
	}
}

/// An iterator over data frames from a Receiver.
pub struct DataFrameIterator<'a, R, D>
	where R: 'a + Receiver<D> {
	
	inner: &'a mut R
}

impl<'a, R, D> Iterator for DataFrameIterator<'a, R, D>
	where R: 'a + Receiver<D> {

	type Item = WebSocketResult<D>;
	
	/// Get the next data frame from the receiver. Always returns `Some`.
	fn next(&mut self) -> Option<WebSocketResult<D>> {
		Some(self.inner.recv_dataframe())
	}
}

/// An iterator over messages from a Receiver.
pub struct MessageIterator<'a, R, D> 
	where R: 'a + Receiver<D> {

	inner: &'a mut R
}

impl<'a, R, D, M> Iterator for MessageIterator<'a, R, D>
	where R: 'a + Receiver<D, Message = M>, M: Message<D> {
	
	type Item = WebSocketResult<M>;
	
	/// Get the next message from the receiver. Always returns `Some`.
	fn next(&mut self) -> Option<WebSocketResult<M>> {
		Some(self.inner.recv_message())
	}
}