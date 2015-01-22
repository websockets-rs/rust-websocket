//! The default implementation of a WebSocket Receiver.

use common::{WebSocketDataFrame, WebSocketMessage};
use common::{Local, Remote};
use result::WebSocketResult;
use ws::Receiver;
use ws::util::dataframe::read_dataframe;

/// A Receiver that wraps a Reader and provides a default implementation using
/// WebSocketDataFrames and WebSocketMessages.
pub struct WebSocketReceiver<R, L> {
	inner: R,
	buffer: Vec<WebSocketDataFrame>
}

impl<R, L> WebSocketReceiver<R, L> {
	/// Create a new WebSocketReceiver using the specified Reader.
	pub fn new(reader: R) -> WebSocketReceiver<R, L> {
		WebSocketReceiver {
			inner: reader,
			buffer: Vec::new()
		}
	}
}

impl<R: Reader> Receiver<WebSocketDataFrame> for WebSocketReceiver<R, Local> {
	type Message = WebSocketMessage;
	
	fn recv_dataframe(&mut self) -> WebSocketResult<WebSocketDataFrame> {
		match self.buffer.pop() {
			Some(dataframe) => Ok(dataframe),
			None => read_dataframe(&mut self.inner, false),
		}
	}
	fn recv_message(&mut self) -> WebSocketResult<WebSocketMessage> {
		panic!();
	}
}

impl<R: Reader> Receiver<WebSocketDataFrame> for WebSocketReceiver<R, Remote> {
	type Message = WebSocketMessage;
	
	fn recv_dataframe(&mut self) -> WebSocketResult<WebSocketDataFrame> {
		match self.buffer.pop() {
			Some(dataframe) => Ok(dataframe),
			None => read_dataframe(&mut self.inner, true),
		}
	}
	fn recv_message(&mut self) -> WebSocketResult<WebSocketMessage> {
		panic!();
	}
}