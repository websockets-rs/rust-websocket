#[phase(plugin)]
extern crate regex_macros;
extern crate regex;
extern crate url;

use super::util::{ReadUntilStr, HeaderCollection, ReadHttpHeaders, WriteHttpHeaders};
use super::check::CheckWebSocketHeader;
use url::{Url, ParseResult, ParseError};
use std::rand;
use std::io::{Reader, Writer, IoResult, IoError, IoErrorKind};
use serialize::base64::{ToBase64, STANDARD};
use std::clone::Clone;

/// Represents a WebSocket handshake request, which is sent from the client to the server.
/// Use the new() function to create a new request.
pub struct WebSocketRequest {
	/// The resource name of the request. E.g. /path/to/resource for the URI ws://www.example.com/path/to/resource
	pub resource_name: String,
	/// The collection of headers contained in this request
	pub headers: HeaderCollection,
}

impl WebSocketRequest {
	/// Creates a new WebSocket handshake request for use with WebSocketClient::connect().
	pub fn new(uri: &str, protocol: &str) -> ParseResult<WebSocketRequest> {
		let ws_uri = match Url::parse(uri) {
			Ok(uri) => { uri }
			Err(e) => { return Err(e); }
		};
		
		match ws_uri.scheme.as_slice() {
			"ws" => {
				
			}
			"wss" => {
				
			}
			_ => { return Err(ParseError::InvalidScheme); }
		}
		
		let host = match ws_uri.serialize_host() {
			Some(host) => { host }
			None => { return Err(ParseError::InvalidCharacter); }
		} + match ws_uri.port_or_default() {
			Some(port) => { ":".to_string() + port.to_string() }
			None => { return Err(ParseError::InvalidCharacter); }
		};

		let resource_name = match ws_uri.serialize_path() {
			Some(resource_name) => {
				resource_name
			}
			None => { return Err(ParseError::InvalidCharacter); }
		};
		
		//Generate random key
		let mut raw_key = [0u8, ..16];
		for i in range(0, 16) {
			raw_key[i] = rand::random::<u8>();
		}
		
		//Serialize key as Base64
		let key = raw_key.to_base64(STANDARD);
		
		let mut headers = HeaderCollection::new();
		headers.insert("Host", host);
		headers.insert("Upgrade", "websocket");
		headers.insert("Connection", "Upgrade");
		
		headers.insert("Sec-WebSocket-Key", key);
		headers.insert("Sec-WebSocket-Version", "13");
		headers.insert("Sec-WebSocket-Protocol", protocol);
		
		Ok(WebSocketRequest {
			resource_name: resource_name,
			headers: headers,
		})
	}
}

impl Clone for WebSocketRequest {
	fn clone(&self) -> WebSocketRequest {
		WebSocketRequest {
			resource_name: self.resource_name.clone(),
			headers: self.headers.clone(),
		}
	}
}

pub trait ReadWebSocketRequest {
	fn read_websocket_request(&mut self) -> IoResult<WebSocketRequest>;
}

impl<R: Reader> ReadWebSocketRequest for R {
	fn read_websocket_request(&mut self) -> IoResult<WebSocketRequest> {
		let request_line = try!(self.read_until_str("\r\n", false));
		
		let re = regex!(r"GET (.*) HTTP/\d+\.?\d*");
		let captures = re.captures(request_line.as_slice()).unwrap();
		
		let resource_name = captures.at(1);
		let headers = try!(self.read_http_headers());
		
		if headers.check_request() {
			Ok(WebSocketRequest {
				resource_name: resource_name.to_string(),
				headers: headers,
			})
		}
		else {
			Err(IoError {
				kind: IoErrorKind::InvalidInput,
				desc: "Invalid headers received - malformed WebSocket header",
				detail: None,
			})
		}
	}
}

pub trait WriteWebSocketRequest {
	fn write_websocket_request(&mut self, request: &WebSocketRequest) -> IoResult<()>;
}

impl<W: Writer> WriteWebSocketRequest for W {
	fn write_websocket_request(&mut self, request: &WebSocketRequest) -> IoResult<()> {
		if !request.headers.check_request() {
			return Err(IoError {
				kind: IoErrorKind::InvalidInput,
				desc: "Invalid headers received - malformed WebSocket header",
				detail: None,
			});
		}
		let request_line = "GET ".to_string() + request.resource_name + " HTTP/1.1".to_string();
		
		try!(self.write_str(request_line.as_slice()));
		try!(self.write_str("\r\n"));
		try!(self.write_http_headers(&(request.headers)));
		try!(self.write_str("\r\n"));
		
		Ok(())
	}
}