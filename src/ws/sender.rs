//! Provides a trait for sending data frames and messages.
//!
//! See the `ws` module documentation for more information.

use ws::Message;
use result::WebSocketResult;

/// A trait for sending data frames and messages.
pub trait Sender<D> {
	/// Sends a single data frame using this sender.
	fn send_dataframe(&mut self, dataframe: D) -> WebSocketResult<()>;
	
	/// Sends a single message using this sender.
	fn send_message<M>(&mut self, message: M) -> WebSocketResult<()> 
		where M: Message<D>, <M as Message<D>>::DataFrameIterator: Iterator<Item = D> {
		// FIXME: Shouldn't need to have the second part of the where clause (#20890)
		
		for dataframe in message.into_iter() {
			try!(self.send_dataframe(dataframe));
		}
		Ok(())
	}
}