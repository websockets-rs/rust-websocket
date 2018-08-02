//! Provides a trait for sending data frames and messages.
//!
//! See the `ws` module documentation for more information.

use result::WebSocketResult;
use std::io::Write;
use ws::dataframe::DataFrame;
use ws::Message;

/// A trait for sending data frames and messages.
pub trait Sender {
	/// Should the messages sent be masked.
	/// See the [RFC](https://tools.ietf.org/html/rfc6455#section-5.3)
	/// for more detail.
	fn is_masked(&self) -> bool;

	/// Sends a single data frame using this sender.
	fn send_dataframe<D, W>(&mut self, writer: &mut W, dataframe: &D) -> WebSocketResult<()>
	where
		D: DataFrame,
		W: Write,
	{
		dataframe.write_to(writer, self.is_masked())?;
		Ok(())
	}

	/// Sends a single message using this sender.
	fn send_message<M, W>(&mut self, writer: &mut W, message: &M) -> WebSocketResult<()>
	where
		M: Message,
		W: Write,
	{
		message.serialize(writer, self.is_masked())?;
		Ok(())
	}
}
