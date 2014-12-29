#![unstable]
//! Structs for sending fragmented WebSocket messages

use common::WebSocketResult;
use dataframe::{WebSocketDataFrame, DataFrameSender, WebSocketOpcode};
use std::path::BytesContainer;
use std::sync::MutexGuard;

/// Struct for sending fragmented text messages.
/// 
/// Obtain from  ```WebSocketClient::frag_send_text()```.
/// You must call ```finish()``` to complete the message (if you don't, the WebSocket stream will become invalid)
pub struct TextFragmentSender<'a, S: DataFrameSender<W>, W: Writer + Send> {
	guard: MutexGuard<'a, S>,
}

/// Struct for sending fragmented binary messages.
/// 
/// Obtain from  ```WebSocketClient::frag_send_bytes()```.
/// You must call ```finish()``` to complete the message (if you don't, the WebSocket stream will become invalid)
pub struct BinaryFragmentSender<'a, S: DataFrameSender<W>, W: Writer + Send> {
	guard: MutexGuard<'a, S>,
}

impl<'a, S: DataFrameSender<W>, W: Writer + Send> TextFragmentSender<'a, S, W> {
	/// Create a new TextFragmentSender
	pub fn new<T: ToString>(mut guard: MutexGuard<'a, S>, text: T) -> WebSocketResult<TextFragmentSender<'a, S, W>> {
		let dataframe = WebSocketDataFrame::new(false, WebSocketOpcode::Text, text.to_string().into_bytes());
		try!(guard.send_dataframe(&dataframe));
		Ok(TextFragmentSender {
			guard: guard,
		})
	}
	/// Send a text fragment immediately
	pub fn send<T: ToString>(&mut self, text: T) -> WebSocketResult<()> {
		let dataframe = WebSocketDataFrame::new(false, WebSocketOpcode::Continuation, text.to_string().into_bytes());
		self.guard.send_dataframe(&dataframe)
	}
	/// Send this fragment and complete the message
	pub fn finish<T: ToString>(mut self, text: T) -> WebSocketResult<()> {
		let dataframe = WebSocketDataFrame::new(true, WebSocketOpcode::Continuation, text.to_string().into_bytes());
		self.guard.send_dataframe(&dataframe)
	}
}

impl<'a, S: DataFrameSender<W>, W: Writer + Send> BinaryFragmentSender<'a, S, W> {
	/// Create a new BinaryFragmentSender
	pub fn new<T: BytesContainer>(mut guard: MutexGuard<'a, S>, data: T) -> WebSocketResult<BinaryFragmentSender<'a, S, W>> {
		let dataframe = WebSocketDataFrame::new(false, WebSocketOpcode::Binary, Vec::new() + data.container_as_bytes());
		try!(guard.send_dataframe(&dataframe));
		Ok(BinaryFragmentSender {
			guard: guard,
		})
	}
	/// Send a binary fragment immediately
	pub fn send<T: BytesContainer>(&mut self,  data: T) -> WebSocketResult<()> {
		let dataframe = WebSocketDataFrame::new(false, WebSocketOpcode::Continuation, Vec::new() + data.container_as_bytes());
		self.guard.send_dataframe(&dataframe)
	}
	/// Send this fragment and complete the message
	pub fn finish<T: BytesContainer>(mut self, data: T) -> WebSocketResult<()> {
		let dataframe = WebSocketDataFrame::new(true, WebSocketOpcode::Continuation, Vec::new() + data.container_as_bytes());
		self.guard.send_dataframe(&dataframe)
	}
}