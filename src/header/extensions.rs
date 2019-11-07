//! Provides the Sec-WebSocket-Extensions header.

use crate::result::{WebSocketError, WebSocketResult};
use hyper;
use hyper::header::parsing::{fmt_comma_delimited, from_comma_delimited};
use hyper::header::{Header, HeaderFormat};
use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

const INVALID_EXTENSION: &str = "Invalid Sec-WebSocket-Extensions extension name";

// TODO: check if extension name is valid according to spec

/// Represents a Sec-WebSocket-Extensions header
#[derive(PartialEq, Clone, Debug)]
pub struct WebSocketExtensions(pub Vec<Extension>);

impl Deref for WebSocketExtensions {
	type Target = Vec<Extension>;

	fn deref(&self) -> &Vec<Extension> {
		&self.0
	}
}

#[derive(PartialEq, Clone, Debug)]
/// A WebSocket extension
pub struct Extension {
	/// The name of this extension
	pub name: String,
	/// The parameters for this extension
	pub params: Vec<Parameter>,
}

impl Extension {
	/// Creates a new extension with the given name
	pub fn new(name: String) -> Extension {
		Extension {
			name,
			params: Vec::new(),
		}
	}
}

impl FromStr for Extension {
	type Err = WebSocketError;

	fn from_str(s: &str) -> WebSocketResult<Extension> {
		let mut ext = s.split(';').map(str::trim);
		Ok(Extension {
			name: match ext.next() {
				Some(x) => x.to_string(),
				None => return Err(WebSocketError::ProtocolError(INVALID_EXTENSION)),
			},
			params: ext
				.map(|x| {
					let mut pair = x.splitn(1, '=').map(|x| x.trim().to_string());

					Parameter {
						name: pair.next().unwrap(),
						value: pair.next(),
					}
				})
				.collect(),
		})
	}
}

impl fmt::Display for Extension {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.name)?;
		for param in &self.params {
			write!(f, "; {}", param)?;
		}
		Ok(())
	}
}

#[derive(PartialEq, Clone, Debug)]
/// A parameter for an Extension
pub struct Parameter {
	/// The name of this parameter
	pub name: String,
	/// The value of this parameter, if any
	pub value: Option<String>,
}

impl Parameter {
	/// Creates a new parameter with the given name and value
	pub fn new(name: String, value: Option<String>) -> Parameter {
		Parameter { name, value }
	}
}

impl fmt::Display for Parameter {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.name)?;
		if let Some(ref x) = self.value {
			write!(f, "={}", x)?;
		}
		Ok(())
	}
}

impl Header for WebSocketExtensions {
	fn header_name() -> &'static str {
		"Sec-WebSocket-Extensions"
	}

	fn parse_header(raw: &[Vec<u8>]) -> hyper::Result<WebSocketExtensions> {
		from_comma_delimited(raw).map(WebSocketExtensions)
	}
}

impl HeaderFormat for WebSocketExtensions {
	fn fmt_header(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		let WebSocketExtensions(ref value) = *self;
		fmt_comma_delimited(fmt, &value[..])
	}
}

impl fmt::Display for WebSocketExtensions {
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
	fn test_header_extensions() {
		use crate::header::Headers;
		let value = vec![b"foo, bar; baz; qux=quux".to_vec()];
		let extensions: WebSocketExtensions = Header::parse_header(&value[..]).unwrap();

		let mut headers = Headers::new();
		headers.set(extensions);

		assert_eq!(
			&headers.to_string()[..],
			"Sec-WebSocket-Extensions: foo, bar; baz; qux=quux\r\n"
		);
	}

	#[bench]
	fn bench_header_extensions_parse(b: &mut test::Bencher) {
		let value = vec![b"foo, bar; baz; qux=quux".to_vec()];
		b.iter(|| {
			let mut extensions: WebSocketExtensions = Header::parse_header(&value[..]).unwrap();
			test::black_box(&mut extensions);
		});
	}

	#[bench]
	fn bench_header_extensions_format(b: &mut test::Bencher) {
		let value = vec![b"foo, bar; baz; qux=quux".to_vec()];
		let val: WebSocketExtensions = Header::parse_header(&value[..]).unwrap();
		b.iter(|| {
			format!("{}", val);
		});
	}
}
