#[phase(plugin)]
extern crate regex_macros;
extern crate regex;

use super::util::{str_eq_ignore_case, ReadUntilStr, HeaderCollection, ReadHttpHeaders, WriteHttpHeaders};
use super::version::HttpVersion;
use sha1::Sha1;
use serialize::base64::{ToBase64, STANDARD};
use std::fmt::Show;
use std::io::{Reader, Writer, IoResult, IoError, IoErrorKind};
use std::string::ToString;
use std::clone::Clone;

static MAGIC_GUID: &'static str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

/// Represents a WebSocket response which is sent from the server to the client.
/// Use the new() function to create a new response, and send it with the WebSocketClient::send_handshake_response() function.
/// 
/// ```
/// use websocket::handshake::WebSocketResponse;
///
/// let response = WebSocketResponse::new("dGhlIHNhbXBsZSBub25jZQ==", Some("myProtocol"));
/// 
/// assert_eq!(response.accept().unwrap().as_slice(), "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=");
/// assert_eq!(response.upgrade().unwrap().as_slice(), "websocket");
/// // ...
/// ```
/// Note that we can also send WebSocket failure responses.
/// 
/// ```
/// use websocket::handshake::WebSocketResponse;
/// 
/// let response = WebSocketResponse::failure(403, "Forbidden");
/// assert_eq!(response.status_code, 403);
/// assert_eq!(response.reason_phrase.as_slice(), "Forbidden");
/// 
/// // ...
/// ```
pub struct WebSocketResponse {
	/// The HTTP version of this request
	pub http_version: HttpVersion,
	/// The status code of the response (for a successful handshake, this should be 101)
	pub status_code: uint,
	/// The human readable reason phrase for the status code (E.g. Switching Protocols)
	pub reason_phrase: String,
	/// The collection of headers contained in this WebSocket response
	pub headers: HeaderCollection,
}

impl WebSocketResponse {
	/// Create a new WebSocket response based on the base-64 encoded key from a client.
	pub fn new<A: Show>(key: &str, protocol: Option<A>) -> WebSocketResponse {
		let accept = WebSocketResponse::gen_accept(key.to_string());
		
		let status_code = 101;
		let reason_phrase = "Switching Protocols".to_string();
		
		let mut headers = HeaderCollection::new();
		headers.insert("Upgrade", "websocket");
		headers.insert("Connection", "Upgrade");
		headers.insert("Sec-WebSocket-Accept", accept);
		
		match protocol {
			Some(protocol) => { headers.insert("Sec-WebSocket-Protocol", protocol); }
			None => { }
		}
		
		WebSocketResponse {
			http_version: HttpVersion::new(1u8, Some(1u8)),
			status_code: status_code,
			reason_phrase: reason_phrase,
			headers: headers,
		}
	}
	
	/// Create a WebSocket response with a particular status code and reason phrase - generally
	/// used to indicate a handshake failure.
	pub fn failure<A: ToString>(status_code: uint, reason_phrase: A) -> WebSocketResponse {
		WebSocketResponse {
			http_version: HttpVersion::new(1u8, Some(1u8)),
			status_code: status_code,
			reason_phrase: reason_phrase.to_string(),
			headers: HeaderCollection::new(),
		}
	}
	
	/// Short-cut to get the Upgrade field value of this request
	pub fn upgrade(&self) -> Option<String> {
		self.headers.get("Upgrade")
	}
	
	/// Short-cut to get the Connection field value of this request
	pub fn connection(&self) -> Option<String> {
		self.headers.get("Connection")
	}
	
	/// Short-cut to get the Sec-WebSocket-Accept field value of this request
	pub fn accept(&self) -> Option<String> {
		self.headers.get("Sec-WebSocket-Accept")
	}
	
	/// Short-cut to get the Sec-WebSocket-Protocol field value of this request
	pub fn protocol(&self) -> Option<String> {
		self.headers.get("Sec-WebSocket-Protocol")
	}
	
	/// Short-cut to get the Sec-WebSocket-Version field value of this request
	/// This may be present when the handshake fails (it should not appear on a successful
	/// handshake response.
	pub fn version(&self) -> Option<Vec<String>> {
		match self.headers.get("Sec-WebSocket-Version") {
			Some(version) => {
				let mut result: Vec<String> = Vec::new();
				let mut values = version.as_slice().split_str(",");
				for value in values {
					result.push(value.trim().to_string());
				}
				Some(result)
			}
			None => { None }
		}
	}
	
	/// Short-cut to get the Sec-WebSocket-Extensions field value of this request
	pub fn extensions(&self) -> Option<Vec<String>> {
		match self.headers.get("Sec-WebSocket-Extensions") {
			Some(extensions) => {
				let mut result: Vec<String> = Vec::new();
				let mut values = extensions.as_slice().split_str(",");
				for value in values {
					result.push(value.trim().to_string());
				}
				Some(result)
			}
			None => { None }
		}
	}
	
	/// Generates the handshake Sec-WebSocket-Accept value from
	/// the given Sec-WebSocket-Key.
	pub fn gen_accept<A: ToString>(key: A) -> String {
		let concat_key = key.to_string() + MAGIC_GUID;
		let mut sha1 = Sha1::new();
		sha1.update(concat_key.into_bytes().as_slice());
		sha1.digest().as_slice().to_base64(STANDARD)
	}
	
	/// Returns true if this response indicates a successful handshake
	/// 
	/// The status code must be 101, the WebSocket-Version must be 13,
	/// the Upgrade must be 'websocket', the Connection must be 'Upgrade',
	/// the Sec-WebSocket-Accept header must match the expected value from
	/// the given key.
	pub fn is_successful<A: ToString>(&self, key: A) -> bool {
		self.status_code == 101 && 
		match self.upgrade() {
			Some(upgrade) => { str_eq_ignore_case(upgrade.as_slice(), "websocket") }
			None => { false }
		} && 
		match self.connection() {
			Some(connection) => { str_eq_ignore_case(connection.as_slice(), "Upgrade") }
			None => { false }
		} && 
		match self.accept() {
			Some(accept) => { accept == WebSocketResponse::gen_accept(key) }
			None => { false }
		}
	}
}

impl Clone for WebSocketResponse {
	fn clone(&self) -> WebSocketResponse {
		WebSocketResponse {
			http_version: self.http_version.clone(),
			status_code: self.status_code,
			reason_phrase: self.reason_phrase.clone(),
			headers: self.headers.clone(),
		}
	}
}

pub trait ReadWebSocketResponse {
	fn read_websocket_response(&mut self) -> IoResult<WebSocketResponse>;
}

impl<R: Reader> ReadWebSocketResponse for R {
	fn read_websocket_response(&mut self) -> IoResult<WebSocketResponse> {
		let status_line = try!(self.read_until_str("\r\n", false));
		
		let re = regex!(r"HTTP/(?P<vmaj>\d+)(?:\.(?P<vmin>\d+))? (?P<sc>\d\d\d) (?P<rp>.*)");
		let captures = re.captures(status_line.as_slice()).unwrap();
		
		let version_major: Option<u8> = match captures.name("vmaj") {
			Some(c) => { from_str(c) },
			None => { None }
		};
		let version_minor: Option<u8> = match captures.name("vmin") {
			Some(c) => { from_str(c) },
			None => { None }
		};
		
		let status_code: Option<uint> = match captures.name("sc") {
			Some(c) => { from_str(c) },
			None => { None }
		};
		let reason_phrase = captures.name("rp").to_string();
		let headers = try!(self.read_http_headers());
		
		Ok(WebSocketResponse { 
			http_version: HttpVersion::new(try!(version_major.ok_or(
				IoError {
					kind: IoErrorKind::InvalidInput,
					desc: "No HTTP version",
					detail: None,
				})
			), version_minor),
			status_code: status_code.unwrap(),
			reason_phrase: reason_phrase,
			headers: headers,
		}) 
	}
}

pub trait WriteWebSocketResponse {
	fn write_websocket_response(&mut self, response: &WebSocketResponse) -> IoResult<()>;
}

impl<W: Writer> WriteWebSocketResponse for W {
	fn write_websocket_response(&mut self, response: &WebSocketResponse) -> IoResult<()> {
		let mut status_line = "HTTP/".to_string() + response.http_version.to_string().as_slice();
		status_line = status_line + " " + response.status_code.to_string().as_slice();
		status_line = status_line + " " + response.reason_phrase.to_string().as_slice();
		
		try!(self.write_str(status_line.as_slice()));
		try!(self.write_str("\r\n"));
		try!(self.write_http_headers(&(response.headers)));
		try!(self.write_str("\r\n"));
		
		Ok(())
	}
}