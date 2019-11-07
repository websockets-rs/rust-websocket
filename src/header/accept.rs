use crate::header::WebSocketKey;
use crate::result::{WebSocketError, WebSocketResult};
use hyper;
use hyper::header::parsing::from_one_raw_str;
use hyper::header::{Header, HeaderFormat};
use std::fmt::{self, Debug};
use std::str::FromStr;

use websocket_base::header::WebSocketAccept as WebSocketAcceptLL;

/// Represents a Sec-WebSocket-Accept header
#[derive(PartialEq, Clone, Copy)]
pub struct WebSocketAccept(WebSocketAcceptLL);

impl Debug for WebSocketAccept {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(f)
	}
}

impl FromStr for WebSocketAccept {
	type Err = WebSocketError;

	fn from_str(accept: &str) -> WebSocketResult<WebSocketAccept> {
		Ok(WebSocketAccept(WebSocketAcceptLL::from_str(accept)?))
	}
}

impl WebSocketAccept {
	/// Create a new WebSocketAccept from the given WebSocketKey
	pub fn new(key: &WebSocketKey) -> WebSocketAccept {
		WebSocketAccept(WebSocketAcceptLL::new(&key.0))
	}
	/// Return the Base64 encoding of this WebSocketAccept
	pub fn serialize(&self) -> String {
		self.0.serialize()
	}
}

impl Header for WebSocketAccept {
	fn header_name() -> &'static str {
		"Sec-WebSocket-Accept"
	}

	fn parse_header(raw: &[Vec<u8>]) -> hyper::Result<WebSocketAccept> {
		from_one_raw_str(raw)
	}
}

impl HeaderFormat for WebSocketAccept {
	fn fmt_header(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		write!(fmt, "{}", self.serialize())
	}
}

#[cfg(all(feature = "nightly", test))]
mod tests {
	use super::*;
	use crate::header::{Headers, WebSocketKey};
	use hyper::header::Header;
	use std::str::FromStr;
	use test;

	#[test]
	fn test_header_accept() {
		let key = FromStr::from_str("dGhlIHNhbXBsZSBub25jZQ==").unwrap();
		let accept = WebSocketAccept::new(&key);
		let mut headers = Headers::new();
		headers.set(accept);

		assert_eq!(
			&headers.to_string()[..],
			"Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n"
		);
	}

	#[test]
	fn test_header_from_str() {
		let accept = WebSocketAccept::from_str("YSBzaW1wbGUgc2FtcGwgbm9uY2U=");
		assert!(accept.is_ok()); // 20 bytes

		let accept = WebSocketAccept::from_str("YSBzaG9ydCBub25jZQ==");
		assert!(accept.is_err()); // < 20 bytes

		let accept = WebSocketAccept::from_str("YSByZWFsbHkgbWFsaWNpb3VzIG5vbmNl");
		assert!(accept.is_err()); // > 20 bytes
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
			let mut accept: WebSocketAccept = Header::parse_header(&value[..]).unwrap();
			test::black_box(&mut accept);
		});
	}

	#[bench]
	fn bench_header_accept_format(b: &mut test::Bencher) {
		let value = vec![b"s3pPLMBiTxaQ9kYGzzhZRbK+xOo=".to_vec()];
		let val: WebSocketAccept = Header::parse_header(&value[..]).unwrap();
		b.iter(|| {
			format!("{}", val.serialize());
		});
	}
}
