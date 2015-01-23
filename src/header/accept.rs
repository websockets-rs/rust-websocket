use hyper::header::{Header, HeaderFormat};
use hyper::header::parsing::from_one_raw_str;
use std::fmt::{self, Show};
use std::str::FromStr;
use std::slice::bytes::copy_memory;
use serialize::base64::{ToBase64, FromBase64, STANDARD};
use header::WebSocketKey;
use openssl::crypto::hash::{HashType, hash};

static MAGIC_GUID: &'static str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

/// Represents a Sec-WebSocket-Accept header
#[derive(PartialEq, Clone, Copy)]
pub struct WebSocketAccept([u8; 20]);

impl Show for WebSocketAccept {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "WebSocketAccept({})", self.serialize())
	}
}

impl FromStr for WebSocketAccept {
	fn from_str(accept: &str) -> Option<WebSocketAccept> {
		match accept.from_base64() {
			Ok(vec) => {
				if vec.len() != 20 { return None; }
				let mut array = [0u8; 20];
				copy_memory(&mut array, &vec[]);
				Some(WebSocketAccept(array))
			}
			Err(_) => { None }
		}
	}
}

impl WebSocketAccept {
	/// Create a new WebSocketAccept from the given WebSocketKey
	pub fn new(key: &WebSocketKey) -> WebSocketAccept {
		let serialized = key.serialize();
		let mut concat_key = String::with_capacity(serialized.len() + 36);
		concat_key.push_str(&serialized[]);
		concat_key.push_str(MAGIC_GUID);
		let output = hash(HashType::SHA1, concat_key.as_bytes());
		let mut bytes = [0u8; 20];
		copy_memory(&mut bytes, &output[]);
		WebSocketAccept(bytes)
	}
	/// Return the Base64 encoding of this WebSocketAccept
	pub fn serialize(&self) -> String {
		let WebSocketAccept(accept) = *self;
		accept.to_base64(STANDARD)
	}
}

impl Header for WebSocketAccept {
	fn header_name(_: Option<WebSocketAccept>) -> &'static str {
		"Sec-WebSocket-Accept"
	}

	fn parse_header(raw: &[Vec<u8>]) -> Option<WebSocketAccept> {
		from_one_raw_str(raw)
	}
}

impl HeaderFormat for WebSocketAccept {
	fn fmt_header(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		write!(fmt, "{}", self.serialize())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use test;
	use std::str::FromStr;
	use header::WebSocketKey;
	#[test]
	fn test_websocket_accept() {
		use header::Headers;
		
		let key = FromStr::from_str("dGhlIHNhbXBsZSBub25jZQ==").unwrap();
		let accept = WebSocketAccept::new(&key);
		let mut headers = Headers::new();
		headers.set(accept);
		
		assert_eq!(&headers.to_string()[], "Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n");
	}
	
	#[bench]
	fn bench_header_accept(b: &mut test::Bencher) {
		let key = WebSocketKey::new();
		b.iter(|| {
			let mut accept = WebSocketAccept::new(&key);
			test::black_box(&mut accept);
		});
	}

	#[bench]
	fn bench_header_accept_serialize(b: &mut test::Bencher) {
		let key = WebSocketKey::new();
		b.iter(|| {
			let accept = WebSocketAccept::new(&key);
			let mut serialized = accept.serialize();
			test::black_box(&mut serialized);
		});
	}

	#[bench]
	fn bench_header_serialize_accept(b: &mut test::Bencher) {
		let key = WebSocketKey::new();
		let accept = WebSocketAccept::new(&key);
		b.iter(|| {
			let mut serialized = accept.serialize();
			test::black_box(&mut serialized);
		});
	}
}