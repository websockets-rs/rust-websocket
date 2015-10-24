//! Provides a trait for receiving data frames and messages.
//!
//! Also provides iterators over data frames and messages.
//! See the `ws` module documentation for more information.

use std::marker::PhantomData;
use ws::Message;
use ws::dataframe::DataFrame;
use result::WebSocketResult;

/// A trait for receiving data frames and messages.
pub trait Receiver<F>: Sized
where F: DataFrame {
	/// Reads a single data frame from this receiver.
	fn recv_dataframe(&mut self) -> WebSocketResult<F>;
	/// Returns the data frames that constitute one message.
	fn recv_message_dataframes(&mut self) -> WebSocketResult<Vec<F>>;

	/// Returns an iterator over incoming data frames.
	fn incoming_dataframes<'a>(&'a mut self) -> DataFrameIterator<'a, Self, F> {
		DataFrameIterator {
			inner: self,
			_dataframe: PhantomData
		}
	}
	/// Reads a single message from this receiver.
	fn recv_message<'m, D, M, I>(&mut self) -> WebSocketResult<M>
	where M: Message<'m, D, DataFrameIterator = I>,
          I: Iterator<Item = D>,
          D: DataFrame
    {
		let dataframes = try!(self.recv_message_dataframes());
		Message::from_dataframes(dataframes)
	}

	/// Returns an iterator over incoming messages.
	fn incoming_messages<'a, M, D>(&'a mut self) -> MessageIterator<'a, Self, D, F, M>
	where M: Message<'a, D>, D: DataFrame {
		MessageIterator {
			inner: self,
			_dataframe: PhantomData,
            _receiver: PhantomData,
			_message: PhantomData,
		}
	}
}

/// An iterator over data frames from a Receiver.
pub struct DataFrameIterator<'a, R, D>
where R: 'a + Receiver<D>, D: DataFrame {
	inner: &'a mut R,
	_dataframe: PhantomData<D>
}

impl<'a, R, D> Iterator for DataFrameIterator<'a, R, D>
where R: 'a + Receiver<D>, D: DataFrame {

	type Item = WebSocketResult<D>;

	/// Get the next data frame from the receiver. Always returns `Some`.
	fn next(&mut self) -> Option<WebSocketResult<D>> {
		Some(self.inner.recv_dataframe())
	}
}

/// An iterator over messages from a Receiver.
pub struct MessageIterator<'a, R, D, F, M>
where R: 'a + Receiver<F>,
      M: Message<'a, D>,
      D: DataFrame,
      F: DataFrame,
{
	inner: &'a mut R,
	_dataframe: PhantomData<D>,
	_message: PhantomData<M>,
    _receiver: PhantomData<F>,
}

impl<'a, R, D, F, M, I> Iterator for MessageIterator<'a, R, D, F, M>
where R: 'a + Receiver<F>,
      M: Message<'a, D, DataFrameIterator = I>,
	  I: Iterator<Item = D>,
	  D: DataFrame,
      F: DataFrame,
{
	type Item = WebSocketResult<M>;

	/// Get the next message from the receiver. Always returns `Some`.
	fn next(&mut self) -> Option<WebSocketResult<M>> {
		Some(self.inner.recv_message())
	}
}
