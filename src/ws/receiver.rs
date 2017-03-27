//! Provides a trait for receiving data frames and messages.
//!
//! Also provides iterators over data frames and messages.
//! See the `ws` module documentation for more information.

use std::io::Read;
use std::marker::PhantomData;
use ws::Message;
use ws::dataframe::DataFrame;
use result::WebSocketResult;

// TODO: maybe this is not needed anymore
/// A trait for receiving data frames and messages.
pub trait Receiver: Sized
{
    type F: DataFrame;

	  /// Reads a single data frame from this receiver.
	  fn recv_dataframe<R>(&mut self, reader: &mut R) -> WebSocketResult<Self::F>
        where R: Read;

	  /// Returns the data frames that constitute one message.
	  fn recv_message_dataframes<R>(&mut self, reader: &mut R) -> WebSocketResult<Vec<Self::F>>
        where R: Read;

	  /// Returns an iterator over incoming data frames.
	  fn incoming_dataframes<'a, R>(&'a mut self, reader: &'a mut R) -> DataFrameIterator<'a, Self, R>
        where R: Read,
    {
		    DataFrameIterator {
            reader: reader,
            inner: self,
		    }
	  }

	  /// Reads a single message from this receiver.
	  fn recv_message<'m, D, M, I, R>(&mut self, reader: &mut R) -> WebSocketResult<M>
	      where M: Message<'m, D, DataFrameIterator = I>,
	            I: Iterator<Item = D>,
	            D: DataFrame,
	            R: Read,
	  {
		    let dataframes = try!(self.recv_message_dataframes(reader));
		    Message::from_dataframes(dataframes)
	  }

	  /// Returns an iterator over incoming messages.
	  fn incoming_messages<'a, M, D, R>(&'a mut self, reader: &'a mut R) -> MessageIterator<'a, Self, D, Self::F, M, R>
	      where M: Message<'a, D>,
              D: DataFrame,
              R: Read,
	  {
		    MessageIterator {
            reader: reader,
			      inner: self,
			      _dataframe: PhantomData,
			      _receiver: PhantomData,
			      _message: PhantomData,
		    }
	  }
}

/// An iterator over data frames from a Receiver.
pub struct DataFrameIterator<'a, Recv, R>
    where Recv: 'a + Receiver,
          R: 'a + Read,
{
    reader: &'a mut R,
    inner: &'a mut Recv,
}

impl<'a, Recv, R> Iterator for DataFrameIterator<'a, Recv, R>
    where Recv: 'a + Receiver,
          R: Read,
{

	  type Item = WebSocketResult<Recv::F>;

	  /// Get the next data frame from the receiver. Always returns `Some`.
	  fn next(&mut self) -> Option<WebSocketResult<Recv::F>> {
		    Some(self.inner.recv_dataframe(self.reader))
	  }
}

/// An iterator over messages from a Receiver.
pub struct MessageIterator<'a, Recv, D, F, M, R>
    where Recv: 'a + Receiver,
          M: Message<'a, D>,
          D: DataFrame,
          F: DataFrame,
          R: 'a + Read,
{
    reader: &'a mut R,
	  inner: &'a mut Recv,
	  _dataframe: PhantomData<D>,
	  _message: PhantomData<M>,
    _receiver: PhantomData<F>,
}

impl<'a, Recv, D, F, M, I, R> Iterator for MessageIterator<'a, Recv, D, F, M, R>
    where Recv: 'a + Receiver,
          M: Message<'a, D, DataFrameIterator = I>,
	        I: Iterator<Item = D>,
	        D: DataFrame,
          F: DataFrame,
          R: Read,
{
	  type Item = WebSocketResult<M>;

	  /// Get the next message from the receiver. Always returns `Some`.
	  fn next(&mut self) -> Option<WebSocketResult<M>> {
		    Some(self.inner.recv_message(self.reader))
	  }
}
