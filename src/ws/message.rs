//! Provides a trait for WebSocket messages
//!
//! See the `ws` module documentation for more information.

use result::WebSocketResult;
use dataframe::DataFrame;

/// A trait for WebSocket messages
pub trait Message<D>: Sized {
	/// An iterator over data frames.
	type DataFrameIntoIterator: Iterator<Item = D>;

	type DataFrameIterator: Iterator<Item: DataFrame>;
	/// Attempt to form a message from a slice of data frames.
	fn from_dataframes(frames: Vec<D>) -> WebSocketResult<Self>;
	/// Turns this message into an iterator over data frames
	fn into_iter(self) -> Self::DataFrameIntoIterator;

	fn iter(&self) -> Self::DataFrameIterator;
}
