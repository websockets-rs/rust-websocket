use hyper::header::{Header, HeaderFormat};
use hyper::header::parsing::{from_one_comma_delimited, fmt_comma_delimited};
use std::fmt;
use std::ops::Deref;

/// Represents a Sec-WebSocket-Extensions header
#[derive(PartialEq, Clone, Debug)]
#[stable]
pub struct WebSocketExtensions(pub Vec<String>);

#[stable]
impl Deref for WebSocketExtensions {
	type Target = Vec<String>;
	#[stable]
    fn deref<'a>(&'a self) -> &'a Vec<String> {
        &self.0
    }
}

impl Header for WebSocketExtensions {
	fn header_name() -> &'static str {
		"Sec-WebSocket-Extensions"
	}

	fn parse_header(raw: &[Vec<u8>]) -> Option<WebSocketExtensions> {
		let extensions = raw.iter()
			.filter_map(|line| from_one_comma_delimited(&line[]))
			.collect::<Vec<Vec<String>>>()
			.concat();
		if extensions.len() > 0 {
			Some(WebSocketExtensions(extensions))
		}
		else {
			None
		}
	}
}

impl HeaderFormat for WebSocketExtensions {
	fn fmt_header(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		let WebSocketExtensions(ref value) = *self;
		fmt_comma_delimited(fmt, &value[])
	}
}
#[cfg(test)]
mod tests {
	use super::*;
	use hyper::header::{Header, HeaderFormatter};
	use test;
	#[test]
	fn test_header_extensions() {
		use header::Headers;
		
		let extensions = WebSocketExtensions(vec!["foo".to_string(), "bar".to_string()]);
		let mut headers = Headers::new();
		headers.set(extensions);
		
		assert_eq!(&headers.to_string()[], "Sec-WebSocket-Extensions: foo, bar\r\n");
	}
	#[bench]
	fn bench_header_extensions_parse(b: &mut test::Bencher) {
		let value = vec![b"foo, bar".to_vec()];
		b.iter(|| {
			let mut extensions: WebSocketExtensions = Header::parse_header(&value[]).unwrap();
			test::black_box(&mut extensions);
		});
	}
	#[bench]
	fn bench_header_extensions_format(b: &mut test::Bencher) {
		let value = vec![b"foo, bar".to_vec()];
		let val: WebSocketExtensions = Header::parse_header(&value[]).unwrap();
		let fmt = HeaderFormatter(&val);
		b.iter(|| {
			format!("{}", fmt);
		});
	}
}