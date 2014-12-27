use hyper::header::{Header, HeaderFormat};
use hyper::header::common::util::{from_one_comma_delimited, fmt_comma_delimited};
use std::fmt::{mod};

/// Represents a Sec-WebSocket-Extensions header
#[deriving(PartialEq, Clone, Show)]
pub struct WebSocketExtensions(pub Vec<String>);

impl Deref<Vec<String>> for WebSocketExtensions {
    fn deref<'a>(&'a self) -> &'a Vec<String> {
        &self.0
    }
}

impl Header for WebSocketExtensions {
	fn header_name(_: Option<WebSocketExtensions>) -> &'static str {
		"Sec-WebSocket-Extensions"
	}

	fn parse_header(raw: &[Vec<u8>]) -> Option<WebSocketExtensions> {
		let extensions = raw.iter()
			.filter_map(|line| from_one_comma_delimited(line.as_slice()))
			.collect::<Vec<Vec<String>>>()
			.concat_vec();
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
		fmt_comma_delimited(fmt, value.as_slice())
	}
}
