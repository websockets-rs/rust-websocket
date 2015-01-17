use hyper::header::{Header, HeaderFormat};
use hyper::header::shared::util::{from_one_comma_delimited, fmt_comma_delimited};
use std::fmt;
use std::ops::Deref;

/// Represents a Sec-WebSocket-Extensions header
#[derive(PartialEq, Clone, Show)]
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
	fn header_name(_: Option<WebSocketExtensions>) -> &'static str {
		"Sec-WebSocket-Extensions"
	}

	fn parse_header(raw: &[Vec<u8>]) -> Option<WebSocketExtensions> {
		let extensions = raw.iter()
			.filter_map(|line| from_one_comma_delimited(line.as_slice()))
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
		fmt_comma_delimited(fmt, value.as_slice())
	}
}

#[test]
fn test_websocket_extensions() {
	use header::Headers;
	
	let extensions = WebSocketExtensions(vec!["foo".to_string(), "bar".to_string()]);
	let mut headers = Headers::new();
	headers.set(extensions);
	
	assert_eq!(&headers.to_string()[], "Sec-WebSocket-Extensions: foo, bar\r\n");
}