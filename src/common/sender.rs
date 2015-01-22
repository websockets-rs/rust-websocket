//! The default implementation of a WebSocket Sender.

use common::WebSocketDataFrame;
use result::WebSocketResult;
use ws::Sender;
use ws::util::dataframe::write_dataframe;

/// A Sender that wraps a Writer and provides a default implementation using
/// WebSocketDataFrames and WebSocketMessages.
pub struct WebSocketSender<W> {
	inner: W,
	local: bool
}

impl<W> WebSocketSender<W> {
	/// Create a new WebSocketSender using the specified Writer.
	pub fn new(writer: W, local: bool) -> WebSocketSender<W> {
		WebSocketSender {
			inner: writer,
			local: local
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

impl<W: Writer> Sender<WebSocketDataFrame> for WebSocketSender<W> {
	fn send_dataframe(&mut self, dataframe: WebSocketDataFrame) -> WebSocketResult<()> {
		write_dataframe(&mut self.inner, self.local, dataframe)
	}
}