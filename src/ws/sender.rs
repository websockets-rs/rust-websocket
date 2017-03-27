//! Provides a trait for sending data frames and messages.
//!
//! See the `ws` module documentation for more information.

use std::io::Write;
use ws::Message;
use ws::dataframe::DataFrame;
use result::WebSocketResult;

// TODO: maybe this is not needed anymore
/// A trait for sending data frames and messages.
pub trait Sender {
	/// Sends a single data frame using this sender.
	fn send_dataframe<D, W>(&mut self, writer: &mut W, dataframe: &D) -> WebSocketResult<()>
	      where D: DataFrame,
              W: Write;

	/// Sends a single message using this sender.
	fn send_message<'m, M, D, W>(&mut self, writer: &mut W, message: &'m M) -> WebSocketResult<()>
	      where M: Message<'m, D>,
              D: DataFrame,
              W: Write,
	{
		for ref dataframe in message.dataframes() {
			try!(self.send_dataframe(writer, dataframe));
		}
		Ok(())
	}
}
