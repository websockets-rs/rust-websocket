//! Provides a trait for WebSocket messages
//!
//! See the `ws` module documentation for more information.

use result::WebSocketResult;
use ws::dataframe::DataFrame;

/// A trait for WebSocket messages
pub trait Message<I, F>: Sized
where I: Iterator<Item = F>, F: DataFrame {
	/// Attempt to form a message from a slice of data frames.
	fn from_dataframes<D>(frames: Vec<D>) -> WebSocketResult<Self>
    where D: DataFrame;
	/// Turns this message into an iterator over data frames
	fn dataframes<'a>(&'a self) -> I;
}
