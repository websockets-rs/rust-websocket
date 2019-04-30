//! This file contains some framework-agnostic aspects of WebSocket HTTP headers.

/// WebSocket header names
pub mod names {
    pub const PROTOCOL: &str = "Sec-WebSocket-Protocol";
    pub const ACCEPT: &str = "Sec-WebSocket-Accept";
    pub const EXTENSIONS: &str = "Sec-WebSocket-Extensions";
    pub const KEY: &str = "Sec-WebSocket-Key";
}

extern crate base64;
extern crate sha1;
use self::sha1::Sha1;

use std::fmt::{self, Debug};
use std::mem;
use std::str::FromStr;
use result::{WebSocketResult, WebSocketError};

/// Represents a Sec-WebSocket-Key header.
#[derive(PartialEq, Clone, Copy, Default)]
pub struct WebSocketKey(pub [u8; 16]);

impl Debug for WebSocketKey {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "WebSocketKey({})", self.serialize())
	}
}

impl FromStr for WebSocketKey {
	type Err = WebSocketError;

	fn from_str(key: &str) -> WebSocketResult<WebSocketKey> {
		match base64::decode(key) {
			Ok(vec) => {
				if vec.len() != 16 {
					return Err(WebSocketError::ProtocolError(
						"Sec-WebSocket-Key must be 16 bytes",
					));
				}
				let mut array = [0u8; 16];
				let mut iter = vec.into_iter();
				for i in &mut array {
					*i = iter.next().unwrap();
				}

				Ok(WebSocketKey(array))
			}
			Err(_) => Err(WebSocketError::ProtocolError(
				"Invalid Sec-WebSocket-Accept",
			)),
		}
	}
}

impl WebSocketKey {
	/// Generate a new, random WebSocketKey
	pub fn new() -> WebSocketKey {
		let key: [u8; 16] = unsafe {
			// Much faster than calling random() several times
			mem::transmute(rand::random::<(u64, u64)>())
		};
		WebSocketKey(key)
	}
	/// Return the Base64 encoding of this WebSocketKey
	pub fn serialize(&self) -> String {
		let WebSocketKey(key) = *self;
		base64::encode(&key)
	}
}



static MAGIC_GUID: &'static str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

/// Represents a Sec-WebSocket-Accept header
#[derive(PartialEq, Clone, Copy)]
pub struct WebSocketAccept([u8; 20]);

impl Debug for WebSocketAccept {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "WebSocketAccept({})", self.serialize())
	}
}

impl FromStr for WebSocketAccept {
	type Err = WebSocketError;

	fn from_str(accept: &str) -> WebSocketResult<WebSocketAccept> {
		match base64::decode(accept) {
			Ok(vec) => {
				if vec.len() != 20 {
					return Err(WebSocketError::ProtocolError(
						"Sec-WebSocket-Accept must be 20 bytes",
					));
				}
				let mut array = [0u8; 20];
				let mut iter = vec.into_iter();
				for i in &mut array {
					*i = iter.next().unwrap();
				}
				Ok(WebSocketAccept(array))
			}
			Err(_) => Err(WebSocketError::ProtocolError(
				"Invalid Sec-WebSocket-Accept ",
			)),
		}
	}
}

impl WebSocketAccept {
	/// Create a new WebSocketAccept from the given WebSocketKey
	pub fn new(key: &WebSocketKey) -> WebSocketAccept {
		let serialized = key.serialize();
		let mut concat_key = String::with_capacity(serialized.len() + 36);
		concat_key.push_str(&serialized[..]);
		concat_key.push_str(MAGIC_GUID);
		let mut sha1 = Sha1::new();
		sha1.update(concat_key.as_bytes());
		let bytes = sha1.digest().bytes();
		WebSocketAccept(bytes)
	}
	/// Return the Base64 encoding of this WebSocketAccept
	pub fn serialize(&self) -> String {
		let WebSocketAccept(accept) = *self;
		base64::encode(&accept)
	}
}
