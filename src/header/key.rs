use hyper::header::{Header, HeaderFormat};
use hyper::header::shared::util::from_one_raw_str;
use std::fmt::{self, Show};
use std::rand;
use std::str::FromStr;
use std::slice::bytes::copy_memory;
use serialize::base64::{ToBase64, FromBase64, STANDARD};

/// Represents a Sec-WebSocket-Key header.
#[derive(PartialEq, Clone, Copy)]
#[stable]
pub struct WebSocketKey(pub [u8; 16]);

#[stable]
impl Show for WebSocketKey {
	#[stable]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "WebSocketKey({})", self.serialize())
	}
}

#[stable]
impl FromStr for WebSocketKey {
	#[stable]
	fn from_str(key: &str) -> Option<WebSocketKey> {
		match key.from_base64() {
			Ok(vec) => {
				if vec.len() != 16 { return None; }
				let mut array = [0u8; 16];
				copy_memory(&mut array, vec.as_slice());
				Some(WebSocketKey(array))
			}
			Err(_) => { None }
		}
	}
}

#[stable]
impl WebSocketKey {
	/// Generate a new, random WebSocketKey
	#[stable]
	pub fn new() -> WebSocketKey {
		let mut key = [0u8; 16];
		for item in key.iter_mut() {
			*item = rand::random::<u8>();
		}
		WebSocketKey(key)
	}
	/// Return the Base64 encoding of this WebSocketKey
	#[stable]
	pub fn serialize(&self) -> String {
		let WebSocketKey(key) = *self;
		key.to_base64(STANDARD)
	}
}

impl Header for WebSocketKey {
	fn header_name(_: Option<WebSocketKey>) -> &'static str {
		"Sec-WebSocket-Key"
	}

	fn parse_header(raw: &[Vec<u8>]) -> Option<WebSocketKey> {
		from_one_raw_str(raw)
	}
}

impl HeaderFormat for WebSocketKey {
	fn fmt_header(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		write!(fmt, "{}", self.serialize())
	}
}