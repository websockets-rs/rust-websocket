use base64;
use hyper::header::{Header, HeaderFormat};
use hyper::header::parsing::from_one_raw_str;
use hyper;
use std::fmt::{self, Debug};
use rand;
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
					return Err(WebSocketError::ProtocolError("Sec-WebSocket-Key must be 16 bytes"));
				}
				let mut array = [0u8; 16];
				let mut iter = vec.into_iter();
				for i in &mut array {
					*i = iter.next().unwrap();
				}

				Ok(WebSocketKey(array))
			}
			Err(_) => Err(WebSocketError::ProtocolError("Invalid Sec-WebSocket-Accept")),
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
		use header::Headers;

		let extensions = WebSocketKey([65; 16]);
		let mut headers = Headers::new();
		headers.set(extensions);

		assert_eq!(&headers.to_string()[..],
		           "Sec-WebSocket-Key: QUFBQUFBQUFBQUFBQUFBQQ==\r\n");
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
