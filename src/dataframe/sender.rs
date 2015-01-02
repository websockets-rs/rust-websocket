//! Traits and structs related to sending WebSocket data frames
#![unstable]
use common::{Local, Remote, WebSocketResult}; 
use dataframe::WebSocketDataFrame;
use dataframe::raw::{RawDataFrame, DataFrameLength};
use dataframe::mask::{mask_data, gen_mask};
use std::io::Writer;
use std::sync::{Arc, Mutex};

/// A trait that allows for sending data frames.
/// 
/// A data frame sender contains an inner Writer, on to which data frames are written.
/// Data frame senders convert data frames into raw bytes to be written to a Writer.
/// The function ```RawDataFrame.write()``` writes RawDataFrames to a Writer, allowing
/// the data frame sender to simply convert a WebSocketDataFrame into a RawDataFrame
/// and let the RawDataFrame do the writing.
pub trait DataFrameSender<W: Send>: Send {
	/// Create a new sender from a Writer
	fn new(inner: W) -> Self;
	/// Write a data frame to the inner Writer
	fn send_dataframe(&mut self, dataframe: &WebSocketDataFrame) -> WebSocketResult<()>;
}

/// The default WebSocket data frame sender
pub struct WebSocketSender<W: Send, L> {
	inner: Arc<Mutex<W>>,
}

unsafe impl<W: Send, L: Send>  Send for WebSocketSender<W, L> {}

impl<W: Writer + Send> DataFrameSender<W> for WebSocketSender<W, Local> {
	/// Create a new local WebSocketSender using the specified Writer
	fn new(inner: W) -> WebSocketSender<W, Local> {
		WebSocketSender {
			inner: Arc::new(Mutex::new(inner))
		}
	}
	/// Send a data frame
	fn send_dataframe(&mut self, dataframe: &WebSocketDataFrame) -> WebSocketResult<()> {
		let masking_key = gen_mask();
		let rawframe = RawDataFrame {
			finished: dataframe.finished,
			reserved: dataframe.reserved,
			opcode: dataframe.opcode,
			mask: Some(masking_key),
			length: DataFrameLength::new(dataframe.data.len()),
			data: mask_data(masking_key, dataframe.data.as_slice()),
		};
		let mut writer = self.inner.lock().unwrap();
		rawframe.write(&mut *writer)
	}
}

impl<W: Writer + Send> DataFrameSender<W> for WebSocketSender<W, Remote> {
	/// Create a new local WebSocketSender using the specified Writer
	fn new(inner: W) -> WebSocketSender<W, Remote> {
		WebSocketSender {
			inner: Arc::new(Mutex::new(inner))
		}
	}
	/// Send a data frame
	fn send_dataframe(&mut self, dataframe: &WebSocketDataFrame) -> WebSocketResult<()> {
		let rawframe = RawDataFrame {
			finished: dataframe.finished,
			reserved: dataframe.reserved,
			opcode: dataframe.opcode,
			mask: None,
			length: DataFrameLength::new(dataframe.data.len()),
			data: dataframe.data.clone(),
		};
		let mut writer = self.inner.lock().unwrap();
		rawframe.write(&mut *writer)
	}
}

impl<W: Writer + Send, L: Send> Clone for WebSocketSender<W, L> {
	fn clone(&self) -> WebSocketSender<W, L> {
		WebSocketSender {
			inner: self.inner.clone()
		}
	}
}