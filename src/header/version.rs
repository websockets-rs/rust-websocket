use hyper::header::{Header, HeaderFormat};
use hyper::header::parsing::from_one_raw_str;
use std::fmt::{self, Show};

/// Represents a Sec-WebSocket-Version header
#[derive(PartialEq, Clone)]
pub enum WebSocketVersion {
	/// The version of WebSocket defined in RFC6455
	WebSocket13,
	/// An unknown version of WebSocket
	Unknown(String)
}

impl fmt::Show for WebSocketVersion {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			WebSocketVersion::WebSocket13 => {
				write!(f, "13")
			}
			WebSocketVersion::Unknown(ref value) => {
				write!(f, "{}", value)
			}
		}
	}
}

impl Header for WebSocketVersion {
	fn header_name(_: Option<WebSocketVersion>) -> &'static str {
		"Sec-WebSocket-Version"
	}

	fn parse_header(raw: &[Vec<u8>]) -> Option<WebSocketVersion> {
		from_one_raw_str(raw).map(|s : String|
			match &s[] {
				"13" => { WebSocketVersion::WebSocket13 }
				_ => { WebSocketVersion::Unknown(s) }
			}
		)
	}
}

impl HeaderFormat for WebSocketVersion {
	fn fmt_header(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.fmt(fmt)
	}
}

#[test]
fn test_websocket_version() {
	use header::Headers;
	
	let version = WebSocketVersion::WebSocket13;
	let mut headers = Headers::new();
	headers.set(version);
	
	assert_eq!(&headers.to_string()[], "Sec-WebSocket-Version: 13\r\n");
}