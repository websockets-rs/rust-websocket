use hyper::header::{Header, HeaderFormat};
use hyper::header::shared::util::{from_one_comma_delimited, fmt_comma_delimited};
use std::fmt;
use std::ops::Deref;

/// Represents a Sec-WebSocket-Protocol header
#[derive(PartialEq, Clone, Show)]
#[stable]
pub struct WebSocketProtocol(pub Vec<String>);

#[stable]
impl Deref for WebSocketProtocol {
	type Target = Vec<String>;
	#[stable]
    fn deref<'a>(&'a self) -> &'a Vec<String> {
        &self.0
    }
}

impl Header for WebSocketProtocol {
	fn header_name(_: Option<WebSocketProtocol>) -> &'static str {
		"Sec-WebSocket-Protocol"
	}

	fn parse_header(raw: &[Vec<u8>]) -> Option<WebSocketProtocol> {
		let protocols = raw.iter()
			.filter_map(|line| from_one_comma_delimited(line.as_slice()))
			.collect::<Vec<Vec<String>>>()
			.concat();
		if protocols.len() > 0 {
			Some(WebSocketProtocol(protocols))
		}
		else {
			None
		}
	}
}

impl HeaderFormat for WebSocketProtocol {
	fn fmt_header(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		let WebSocketProtocol(ref value) = *self;
		fmt_comma_delimited(fmt, value.as_slice())
	}
}
