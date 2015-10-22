//! Provides a trait for WebSocket messages
//!
//! See the `ws` module documentation for more information.

use result::WebSocketResult;
use ws::dataframe::DataFrame;

/// A trait for WebSocket messages
pub trait Message<'a, F>: Sized
where F: DataFrame {
	/// The iterator type returned by dataframes
	type DataFrameIterator: Iterator<Item = F>;
	/// Attempt to form a message from a slice of data frames.
	fn from_dataframes<D>(frames: Vec<D>) -> WebSocketResult<Self>
    where D: DataFrame;
	/// Turns this message into an iterator over data frames
	fn dataframes(&'a self) -> Self::DataFrameIterator;
}
