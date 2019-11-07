use hyper;
use hyper::header::parsing::from_one_raw_str;
use hyper::header::{Header, HeaderFormat};
use std::fmt;
use std::ops::Deref;

/// Represents an Origin header
#[derive(PartialEq, Clone, Debug)]
pub struct Origin(pub String);

impl Deref for Origin {
	type Target = String;
	fn deref(&self) -> &String {
		&self.0
	}
}

impl Header for Origin {
	fn header_name() -> &'static str {
		"Origin"
	}

	fn parse_header(raw: &[Vec<u8>]) -> hyper::Result<Origin> {
		from_one_raw_str(raw).map(Origin)
	}
}

impl HeaderFormat for Origin {
	fn fmt_header(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		let Origin(ref value) = *self;
		write!(fmt, "{}", value)
	}
}

impl fmt::Display for Origin {
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
	fn test_header_origin() {
		use crate::header::Headers;

		let origin = Origin("foo bar".to_string());
		let mut headers = Headers::new();
		headers.set(origin);

		assert_eq!(&headers.to_string()[..], "Origin: foo bar\r\n");
	}

	#[bench]
	fn bench_header_origin_parse(b: &mut test::Bencher) {
		let value = vec![b"foobar".to_vec()];
		b.iter(|| {
			let mut origin: Origin = Header::parse_header(&value[..]).unwrap();
			test::black_box(&mut origin);
		});
	}

	#[bench]
	fn bench_header_origin_format(b: &mut test::Bencher) {
		let value = vec![b"foobar".to_vec()];
		let val: Origin = Header::parse_header(&value[..]).unwrap();
		b.iter(|| {
			format!("{}", val);
		});
	}
}
