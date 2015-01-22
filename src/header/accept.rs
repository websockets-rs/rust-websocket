use hyper::header::{Header, HeaderFormat};
use hyper::header::shared::util::from_one_raw_str;
use std::fmt::{self, Show};
use std::hash::Writer;
use sha1::Sha1;
use std::str::FromStr;
use std::slice::bytes::copy_memory;
use serialize::base64::{ToBase64, FromBase64, STANDARD};
use header::WebSocketKey;

static MAGIC_GUID: &'static str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

/// Represents a Sec-WebSocket-Accept header
#[derive(PartialEq, Clone, Copy)]
#[stable]
pub struct WebSocketAccept([u8; 20]);

#[stable]
impl Show for WebSocketAccept {
	#[stable]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "WebSocketAccept({})", self.serialize())
	}
}

#[stable]
impl FromStr for WebSocketAccept {
	#[stable]
	fn from_str(accept: &str) -> Option<WebSocketAccept> {
		match accept.from_base64() {
			Ok(vec) => {
				if vec.len() != 20 { return None; }
				let mut array = [0u8; 20];
				copy_memory(&mut array, vec.as_slice());
				Some(WebSocketAccept(array))
			}
			Err(_) => { None }
		}
	}
}

#[stable]
impl WebSocketAccept {
	/// Create a new WebSocketAccept from the given WebSocketKey
	#[stable]
	pub fn new(key: &WebSocketKey) -> WebSocketAccept {
		let concat_key = key.serialize() + MAGIC_GUID;
		let mut sha1 = Sha1::new();
		sha1.write(concat_key.into_bytes().as_slice());
		let mut bytes = [0u8; 20];
		sha1.output(&mut bytes);
		WebSocketAccept(bytes)
	}
	/// Return the Base64 encoding of this WebSocketAccept
	#[stable]
	pub fn serialize(&self) -> String {
		let WebSocketAccept(accept) = *self;
		accept.to_base64(STANDARD)
	}
}

impl Header for WebSocketAccept {
	fn header_name(_: Option<WebSocketAccept>) -> &'static str {
		"Sec-WebSocket-Accept"
	}

	fn parse_header(raw: &[Vec<u8>]) -> Option<WebSocketAccept> {
		from_one_raw_str(raw)
	}
}

impl HeaderFormat for WebSocketAccept {
	fn fmt_header(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		write!(fmt, "{}", self.serialize())
	}
}

#[test]
fn test_websocket_accept() {
	use header::Headers;
	
	let key = FromStr::from_str("dGhlIHNhbXBsZSBub25jZQ==").unwrap();
	let accept = WebSocketAccept::new(&key);
	let mut headers = Headers::new();
	headers.set(accept);
	
	assert_eq!(&headers.to_string()[], "Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n");
}