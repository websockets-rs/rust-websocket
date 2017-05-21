//! Provides a trait for WebSocket messages
//!
//! See the `ws` module documentation for more information.

use std::io::Write;
use ws::dataframe::DataFrame as DataFrameable;
use result::WebSocketResult;

/// A trait for WebSocket messages
pub trait Message: Sized {
	/// Writes this message to the writer
	fn serialize(&self, &mut Write, masked: bool) -> WebSocketResult<()>;

	/// Returns how many bytes this message will take up
	fn message_size(&self, masked: bool) -> usize;

	/// Attempt to form a message from a series of data frames
	fn from_dataframes<D: DataFrameable>(frames: Vec<D>) -> WebSocketResult<Self>;
}
