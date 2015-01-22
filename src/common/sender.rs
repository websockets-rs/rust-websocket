//! The default implementation of a WebSocket Sender.

use common::WebSocketDataFrame;
use common::{Local, Remote};
use result::WebSocketResult;
use ws::Sender;
use ws::util::dataframe::write_dataframe;

/// A Sender that wraps a Writer and provides a default implementation using
/// WebSocketDataFrames and WebSocketMessages.
pub struct WebSocketSender<W, L> {
	inner: W
}

impl<W, L> WebSocketSender<W, L> {
	/// Create a new WebSocketSender using the specified Writer.
	pub fn new(writer: W) -> WebSocketSender<W, L> {
		WebSocketSender {
			inner: writer
		}
	}
}

impl<W: Writer> Sender<WebSocketDataFrame> for WebSocketSender<W, Local> {
	fn send_dataframe(&mut self, dataframe: WebSocketDataFrame) -> WebSocketResult<()> {
		write_dataframe(&mut self.inner, true, dataframe)
	}
}

impl<W: Writer> Sender<WebSocketDataFrame> for WebSocketSender<W, Remote> {
	fn send_dataframe(&mut self, dataframe: WebSocketDataFrame) -> WebSocketResult<()> {
		write_dataframe(&mut self.inner, false, dataframe)
	}
}