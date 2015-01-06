//! Provides an iterator over incoming data frames and messages
#![unstable]
use dataframe::sender::DataFrameSender;
use dataframe::receiver::DataFrameReceiver;
use dataframe::converter::DataFrameConverter;
use dataframe::WebSocketDataFrame;
use message::WebSocketMessaging;
use client::WebSocketClient;
use common::WebSocketResult;
use std::option::Option;
use std::iter::Iterator;

/// An iterator over incoming data frames. Always returns Some().
#[unstable]
pub struct IncomingDataFrames<'a, S: 'a, R: 'a, C: 'a> {
	inner: &'a mut WebSocketClient<S, R, C>,
}

/// An iterator over incoming messages. Always returns Some().
#[unstable]
pub struct IncomingMessages<'a, S: 'a, R: 'a, C: 'a> {
	inner: &'a mut WebSocketClient<S, R, C>,
}

impl<'a, S: DataFrameSender<W>, R: DataFrameReceiver<E>, C: DataFrameConverter<M>, E: Reader + Send, W: Writer + Send, M: WebSocketMessaging> IncomingDataFrames<'a, S, R, C> {
	/// Create a new iterator over incoming data frames
	pub fn new(inner: &'a mut WebSocketClient<S, R, C>) -> IncomingDataFrames<S, R, C> {
		IncomingDataFrames {
			inner: inner
		}
	}
}

impl<'a, S: DataFrameSender<W>, R: DataFrameReceiver<E>, C: DataFrameConverter<M>, E: Reader + Send, W: Writer + Send, M: WebSocketMessaging> IncomingMessages<'a, S, R, C> {
	/// Create a new iterator over incoming messages
	pub fn new(inner: &'a mut WebSocketClient<S, R, C>) -> IncomingMessages<S, R, C> {
		IncomingMessages {
			inner: inner
		}
	}
}

impl<'a, S: DataFrameSender<W>, R: DataFrameReceiver<E>, C: DataFrameConverter<M>, E: Reader + Send, W: Writer + Send, M: WebSocketMessaging> Iterator for IncomingDataFrames<'a, S, R, C> {
	type Item = WebSocketResult<WebSocketDataFrame>;
	fn next(&mut self) -> Option<WebSocketResult<WebSocketDataFrame>> {
		Some(self.inner.recv_dataframe())
	}
}

impl<'a, S: DataFrameSender<W>, R: DataFrameReceiver<E>, C: DataFrameConverter<M>, E: Reader + Send, W: Writer + Send, M: WebSocketMessaging> Iterator for IncomingMessages<'a, S, R, C> {
	type Item = WebSocketResult<M>;
	fn next(&mut self) -> Option<WebSocketResult<M>> {
		Some(self.inner.recv_message())
	}
}
