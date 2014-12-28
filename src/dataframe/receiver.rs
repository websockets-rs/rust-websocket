//! Traits and structs related to receiving WebSocket data frames
#![unstable]
use common::{Local, Remote, WebSocketResult, WebSocketError}; 
use dataframe::WebSocketDataFrame;
use dataframe::raw::RawDataFrame;
use dataframe::mask::mask_data;
use std::io::Reader;
use std::sync::{Arc, Mutex};

/// A trait that allows for receiving data frames.
/// 
/// A data frame receiver contains an inner Reader, off of which data frames are read.
/// Data frame receivers read data frames off a Reader, and return WebSocketDataFrames.
/// The function RawDataFrame::read() can be used to read a RawDataFrame from a Reader,
/// enabling the receiver to simply convert that RawDataFrame to a WebSocketDataFrame.
pub trait DataFrameReceiver<R: Reader + Send>: Send {
/// WebSocketDataFrame.
	/// Create a new receiver from a Reader
	fn new(inner: R) -> Self;
	/// Reads a data frame from the inner Reader
	fn recv_dataframe(&mut self) -> WebSocketResult<WebSocketDataFrame>;
}

/// The default WebSocket data frame receiver
#[deriving(Send)]
pub struct WebSocketReceiver<R: Reader, L> {
	inner: Arc<Mutex<R>>,
}

impl<R: Reader + Send> DataFrameReceiver<R> for WebSocketReceiver<R, Local> {
	/// Create a new receiver from a Reader
	fn new(inner: R) -> WebSocketReceiver<R, Local> {
		WebSocketReceiver {
			inner: Arc::new(Mutex::new(inner))
		}
	}
	/// Receive a data frame
	fn recv_dataframe(&mut self) -> WebSocketResult<WebSocketDataFrame> {
		let mut reader = self.inner.lock();
		let rawframe = try!(RawDataFrame::read(&mut *reader));
		if rawframe.mask.is_some() {
			return Err(WebSocketError::DataFrameError("Data frames from the server must not be masked".to_string()));
		}
		Ok(WebSocketDataFrame {
			finished: rawframe.finished,
			reserved: rawframe.reserved,
			opcode: rawframe.opcode,
			data: rawframe.data,
		})
	}
}

impl<R: Reader + Send> DataFrameReceiver<R> for WebSocketReceiver<R, Remote> {
	fn new(inner: R) -> WebSocketReceiver<R, Remote> {
		WebSocketReceiver {
			inner: Arc::new(Mutex::new(inner))
		}
	}
	fn recv_dataframe(&mut self) -> WebSocketResult<WebSocketDataFrame> {
		let mut reader = self.inner.lock();
		let rawframe = try!(RawDataFrame::read(&mut *reader));
		if rawframe.mask.is_none() {
			return Err(WebSocketError::DataFrameError("Data frames from the client must be masked".to_string()));
		}
		Ok(WebSocketDataFrame {
			finished: rawframe.finished,
			reserved: rawframe.reserved,
			opcode: rawframe.opcode,
			data: mask_data(rawframe.mask.unwrap(), rawframe.data.as_slice()),
		})
	}
}

impl<R: Reader, L> Clone for WebSocketReceiver<R, L> {
	fn clone(&self) -> WebSocketReceiver<R, L> {
		WebSocketReceiver {
			inner: self.inner.clone()
		}
	}
}