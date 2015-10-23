//! Utility functions for various portions of Rust-WebSocket.

pub mod header;
pub mod dataframe;
pub mod mask;
pub mod url;

use std::str::from_utf8;
use std::str::Utf8Error;

pub fn bytes_to_string(data: &[u8]) -> Result<String, Utf8Error> {
	let utf8 = try!(from_utf8(data));
	Ok(utf8.to_string())
}
