use hyper;
use hyper::header::parsing::{fmt_comma_delimited, from_comma_delimited};
use hyper::header::{Header, HeaderFormat};
use std::fmt;
use std::ops::Deref;

// TODO: only allow valid protocol names to be added

/// Represents a Sec-WebSocket-Protocol header
#[derive(PartialEq, Clone, Debug)]
pub struct WebSocketProtocol(pub Vec<String>);

impl Deref for WebSocketProtocol {
	type Target = Vec<String>;
	fn deref(&self) -> &Vec<String> {
		&self.0
	}
}

impl Header for WebSocketProtocol {
	fn header_name() -> &'static str {
		"Sec-WebSocket-Protocol"
	}

	fn parse_header(raw: &[Vec<u8>]) -> hyper::Result<WebSocketProtocol> {
		from_comma_delimited(raw).map(WebSocketProtocol)
	}
}

impl HeaderFormat for WebSocketProtocol {
	fn fmt_header(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		let WebSocketProtocol(ref value) = *self;
		fmt_comma_delimited(fmt, &value[..])
	}
}

impl fmt::Display for WebSocketProtocol {
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
	fn test_header_protocol() {
		use crate::header::Headers;

		let protocol = WebSocketProtocol(vec!["foo".to_string(), "bar".to_string()]);
		let mut headers = Headers::new();
		headers.set(protocol);

		assert_eq!(
			&headers.to_string()[..],
			"Sec-WebSocket-Protocol: foo, bar\r\n"
		);
	}

	#[bench]
	fn bench_header_protocol_parse(b: &mut test::Bencher) {
		let value = vec![b"foo, bar".to_vec()];
		b.iter(|| {
			let mut protocol: WebSocketProtocol = Header::parse_header(&value[..]).unwrap();
			test::black_box(&mut protocol);
		});
	}

	#[bench]
	fn bench_header_protocol_format(b: &mut test::Bencher) {
		let value = vec![b"foo, bar".to_vec()];
		let val: WebSocketProtocol = Header::parse_header(&value[..]).unwrap();
		b.iter(|| {
			format!("{}", val);
		});
	}
}
