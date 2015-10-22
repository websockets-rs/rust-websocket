//! The default implementation of a WebSocket Sender.

use std::io::Write;
use dataframe::DataFrame;
use result::WebSocketResult;
use ws::util::dataframe::write_dataframe;
use ws;

/// A Sender that wraps a Writer and provides a default implementation using
/// DataFrames and Messages.
pub struct Sender<W> {
	inner: W
}

impl<W> Sender<W> {
	/// Create a new WebSocketSender using the specified Writer.
	pub fn new(writer: W) -> Sender<W> {
		Sender {
			inner: writer
		}
	}
	/// Returns a reference to the underlying Writer.
	pub fn get_ref(&self) -> &W {
		&self.inner
	}
	/// Returns a mutable reference to the underlying Writer.
	pub fn get_mut(&mut self) -> &mut W {
		&mut self.inner
	}
}

impl<W: Write> ws::Sender<DataFrame> for Sender<W> {
	/// Sends a single data frame to the remote endpoint.
	fn send_dataframe(&mut self, dataframe: &DataFrame) -> WebSocketResult<()> {
		write_dataframe(&mut self.inner, true, dataframe)
	}
}
