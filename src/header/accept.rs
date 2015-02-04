use hyper::header::{Header, HeaderFormat};
use hyper::header::parsing::from_one_raw_str;
use std::fmt::{self, Debug};
use std::str::FromStr;
use std::slice::bytes::copy_memory;
use serialize::base64::{ToBase64, FromBase64, STANDARD};
use header::WebSocketKey;
use openssl::crypto::hash::{self, hash};
use result::{WebSocketResult, WebSocketError};

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
		match accept.from_base64() {
			Ok(vec) => {
				if vec.len() != 20 {
					return Err(WebSocketError::ProtocolError(
						"Sec-WebSocket-Accept must be 20 bytes".to_string()
					));
				}
				let mut array = [0u8; 20];
				copy_memory(&mut array, &vec[]);
				Ok(WebSocketAccept(array))
			}
			Err(_) => {
				return Err(WebSocketError::ProtocolError(
					"Invalid Sec-WebSocket-Accept ".to_string()
				));
			}
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
		let output = hash(hash::Type::SHA1, concat_key.as_bytes());
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
	fn header_name() -> &'static str {
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
	use header::{Headers, WebSocketKey};
	use hyper::header::{Header, HeaderFormatter};
	#[test]
	fn test_header_accept() {
		let key = FromStr::from_str("dGhlIHNhbXBsZSBub25jZQ==").unwrap();
		let accept = WebSocketAccept::new(&key);
		let mut headers = Headers::new();
		headers.set(accept);
		
		assert_eq!(&headers.to_string()[], "Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n");
	}
	#[bench]
	fn bench_header_accept_new(b: &mut test::Bencher) {
		let key = WebSocketKey::new();
		b.iter(|| {
			let mut accept = WebSocketAccept::new(&key);
			test::black_box(&mut accept);
		});
	}
	#[bench]
	fn bench_header_accept_parse(b: &mut test::Bencher) {
		let value = vec![b"s3pPLMBiTxaQ9kYGzzhZRbK+xOo=".to_vec()];
		b.iter(|| {
			let mut accept: WebSocketAccept = Header::parse_header(&value[]).unwrap();
			test::black_box(&mut accept);
		});
	}
	#[bench]
	fn bench_header_accept_format(b: &mut test::Bencher) {
		let value = vec![b"s3pPLMBiTxaQ9kYGzzhZRbK+xOo=".to_vec()];
		let val: WebSocketAccept = Header::parse_header(&value[]).unwrap();
		let fmt = HeaderFormatter(&val);
		b.iter(|| {
			format!("{}", fmt);
		});
	}
}