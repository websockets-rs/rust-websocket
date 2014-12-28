//! Traits and structs related to conversion of data frames into messages
#![unstable]
use common::{WebSocketResult, WebSocketError};
use message::WebSocketMessaging;
use dataframe::opcode::WebSocketOpcode;
use dataframe::WebSocketDataFrame;

/// A trait which allows the conversion of a series of data frames into a series of messages.
/// 
/// Three functions are required for functionality, and WebSocketConverter is the default implementation.
pub trait DataFrameConverter<M: WebSocketMessaging> : Send {
	/// Creates a new data frame converter.
	fn new() -> Self;
	/// Pushes a data frame on to this converter for processing.
	fn push(&mut self, dataframe: WebSocketDataFrame) -> WebSocketResult<()>;
	/// Pushes a number of data frames on to this converter.
	fn push_all(&mut self, dataframes: &[WebSocketDataFrame]) -> WebSocketResult<()> {
		for dataframe in dataframes.iter() {
			try!(self.push(dataframe.clone()));
		}
		Ok(())
	}
	/// Pops a message from this converter if one is available
	fn pop(&mut self) -> Option<M>;
}

/// Default implementation of a converter that takes WebSocketDataFrames and produces WebSocketMessages
#[deriving(Send)]
pub struct WebSocketConverter<M: WebSocketMessaging> {
	opcode: Option<WebSocketOpcode>,
	data: Vec<u8>,
	messages: Vec<M>,
}

impl<M: WebSocketMessaging> DataFrameConverter<M> for WebSocketConverter<M> {
	/// Creates a new DataFrameConverter
	fn new() -> WebSocketConverter<M> {
		WebSocketConverter {
			opcode: None,
			data: Vec::new(),
			messages: Vec::new(),
		}
	}
	/// Push a data frame on to this converter.
	fn push(&mut self, dataframe: WebSocketDataFrame) -> WebSocketResult<()> {
		let (data, opcode) = match dataframe.opcode {
			// This is a continuation, we should already know the opcode
			WebSocketOpcode::Continuation => {(
				// Append the data frame data and our old data
				self.data.clone() + dataframe.data.as_slice(),
				match self.opcode {
					Some(opcode) => opcode,
					None => { return Err(WebSocketError::ProtocolError("Unexpected continuation data frame".to_string())); }
				}
			)}
			// This is the start (and possibly end) of a message
			_ => {
				// If we've just started, but have not finished, we need to record the opcode
				if !dataframe.finished { self.opcode = Some(dataframe.opcode); }
				// Since it's the start, use the data frame data and opcode
				(dataframe.data, dataframe.opcode)
			}
		};
		
		if dataframe.finished {
			let message = try!(WebSocketMessaging::from_data(opcode, data));
			self.messages.push(message);
		}
		else {
			self.data = data;
		}
		
		Ok(())
	}
	
	/// Pop a message from this converter (returns None if no complete message is available).
	fn pop(&mut self) -> Option<M> {
		self.messages.pop()
	}
}