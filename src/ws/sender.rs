//! Provides a trait for sending data frames and messages.
//!
//! See the `ws` module documentation for more information.

use ws::Message;
use ws::dataframe::DataFrame;
use result::WebSocketResult;

/// A trait for sending data frames and messages.
pub trait Sender {
	/// Sends a single data frame using this sender.
	fn send_dataframe<D>(&mut self, dataframe: &D) -> WebSocketResult<()>
	where D: DataFrame;

	/// Sends a single message using this sender.
	fn send_message<'m, M, D>(&mut self, message: &'m M) -> WebSocketResult<()>
	where M: Message<'m, D>, D: DataFrame {
		for ref dataframe in message.dataframes() {
			try!(self.send_dataframe(dataframe));
		}
		Ok(())
	}
}
