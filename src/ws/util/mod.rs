//! Utility functions for various portions of Rust-WebSocket.

pub mod header;
pub mod mask;

use std::str::from_utf8;
use std::str::Utf8Error;

/// Transforms a u8 slice into an owned String
pub fn bytes_to_string(data: &[u8]) -> Result<String, Utf8Error> {
	let utf8 = from_utf8(data)?;
	Ok(utf8.to_string())
}
