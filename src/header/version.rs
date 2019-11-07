use hyper;
use hyper::header::parsing::from_one_raw_str;
use hyper::header::{Header, HeaderFormat};
use std::fmt::{self, Debug};

/// Represents a Sec-WebSocket-Version header
#[derive(PartialEq, Clone)]
pub enum WebSocketVersion {
	/// The version of WebSocket defined in RFC6455
	WebSocket13,
	/// An unknown version of WebSocket
	Unknown(String),
}

impl fmt::Debug for WebSocketVersion {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			WebSocketVersion::WebSocket13 => write!(f, "13"),
			WebSocketVersion::Unknown(ref value) => write!(f, "{}", value),
		}
	}
}

impl Header for WebSocketVersion {
	fn header_name() -> &'static str {
		"Sec-WebSocket-Version"
	}

	fn parse_header(raw: &[Vec<u8>]) -> hyper::Result<WebSocketVersion> {
		from_one_raw_str(raw).map(|s: String| match &s[..] {
			"13" => WebSocketVersion::WebSocket13,
			_ => WebSocketVersion::Unknown(s),
		})
	}
}

impl HeaderFormat for WebSocketVersion {
	fn fmt_header(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.fmt(fmt)
	}
}

impl fmt::Display for WebSocketVersion {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.fmt_header(fmt)
	}
}

#[cfg(all(feature = "nightly", test))]
mod tests {
	use super::*;
	use hyper::header::Header;
	use test;

	#[test]
	fn test_websocket_version() {
		use crate::header::Headers;

		let version = WebSocketVersion::WebSocket13;
		let mut headers = Headers::new();
		headers.set(version);

		assert_eq!(&headers.to_string()[..], "Sec-WebSocket-Version: 13\r\n");
	}

	#[bench]
	fn bench_header_version_parse(b: &mut test::Bencher) {
		let value = vec![b"13".to_vec()];
		b.iter(|| {
			let mut version: WebSocketVersion = Header::parse_header(&value[..]).unwrap();
			test::black_box(&mut version);
		});
	}

	#[bench]
	fn bench_header_version_format(b: &mut test::Bencher) {
		let value = vec![b"13".to_vec()];
		let val: WebSocketVersion = Header::parse_header(&value[..]).unwrap();
		b.iter(|| {
			format!("{}", val);
		});
	}
}
