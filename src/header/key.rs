use crate::result::{WebSocketError, WebSocketResult};
use hyper;
use hyper::header::parsing::from_one_raw_str;
use hyper::header::{Header, HeaderFormat};
use std::fmt::{self, Debug};
use std::str::FromStr;

use websocket_base::header::WebSocketKey as WebSocketKeyLL;

/// Represents a Sec-WebSocket-Key header.
#[derive(PartialEq, Clone, Copy, Default)]
pub struct WebSocketKey(pub WebSocketKeyLL);

impl Debug for WebSocketKey {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(f)
	}
}

impl FromStr for WebSocketKey {
	type Err = WebSocketError;

	fn from_str(key: &str) -> WebSocketResult<WebSocketKey> {
		Ok(WebSocketKey(WebSocketKeyLL::from_str(key)?))
	}
}

impl WebSocketKey {
	/// Generate a new, random WebSocketKey
	pub fn new() -> WebSocketKey {
		WebSocketKey(WebSocketKeyLL::new())
	}
	/// Return the Base64 encoding of this WebSocketKey
	pub fn serialize(&self) -> String {
		self.0.serialize()
	}

	/// Create WebSocketKey by explicitly specifying the key
	pub fn from_array(a: [u8; 16]) -> WebSocketKey {
		WebSocketKey(WebSocketKeyLL(a))
	}
}

impl Header for WebSocketKey {
	fn header_name() -> &'static str {
		"Sec-WebSocket-Key"
	}

	fn parse_header(raw: &[Vec<u8>]) -> hyper::Result<WebSocketKey> {
		from_one_raw_str(raw)
	}
}

impl HeaderFormat for WebSocketKey {
	fn fmt_header(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		write!(fmt, "{}", self.serialize())
	}
}

#[cfg(all(feature = "nightly", test))]
mod tests {
	use super::*;
	use hyper::header::Header;
	use test;

	#[test]
	fn test_header_key() {
		use crate::header::Headers;

		let extensions = WebSocketKey::from_array([65; 16]);
		let mut headers = Headers::new();
		headers.set(extensions);

		assert_eq!(
			&headers.to_string()[..],
			"Sec-WebSocket-Key: QUFBQUFBQUFBQUFBQUFBQQ==\r\n"
		);
	}

	#[test]
	fn test_header_from_str() {
		let key = WebSocketKey::from_str("YSByZWFsbCBnb29kIGtleQ==");
		assert!(key.is_ok()); // 16 bytes

		let key = WebSocketKey::from_str("YSBzaG9ydCBrZXk=");
		assert!(key.is_err()); // < 16 bytes

		let key = WebSocketKey::from_str("YSB2ZXJ5IHZlcnkgbG9uZyBrZXk=");
		assert!(key.is_err()); // > 16 bytes
	}

	#[bench]
	fn bench_header_key_new(b: &mut test::Bencher) {
		b.iter(|| {
			let mut key = WebSocketKey::new();
			test::black_box(&mut key);
		});
	}

	#[bench]
	fn bench_header_key_parse(b: &mut test::Bencher) {
		let value = vec![b"QUFBQUFBQUFBQUFBQUFBQQ==".to_vec()];
		b.iter(|| {
			let mut key: WebSocketKey = Header::parse_header(&value[..]).unwrap();
			test::black_box(&mut key);
		});
	}

	#[bench]
	fn bench_header_key_format(b: &mut test::Bencher) {
		let value = vec![b"QUFBQUFBQUFBQUFBQUFBQQ==".to_vec()];
		let val: WebSocketKey = Header::parse_header(&value[..]).unwrap();
		b.iter(|| {
			format!("{}", val.serialize());
		});
	}
}
