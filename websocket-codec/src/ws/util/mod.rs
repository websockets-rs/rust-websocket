//! Utility functions for various portions of Rust-WebSocket.

pub mod header;
pub mod mask;

use std::str::from_utf8;
use std::str::Utf8Error;

#[cfg(feature = "async")]
use tokio_codec::{Framed, FramedParts};

/// Transforms a u8 slice into an owned String
pub fn bytes_to_string(data: &[u8]) -> Result<String, Utf8Error> {
	let utf8 = from_utf8(data)?;
	Ok(utf8.to_string())
}

/// Updates codec of Framed
#[cfg(feature = "async")]
pub fn update_framed_codec<S, B, A>(framed: Framed<S, B>, codec: A) -> Framed<S, A> {
	let old_parts = framed.into_parts();
	let mut new_parts = FramedParts::new(old_parts.io, codec);
	new_parts.read_buf = old_parts.read_buf;
	new_parts.write_buf = old_parts.write_buf;
	Framed::from_parts(new_parts)
}
