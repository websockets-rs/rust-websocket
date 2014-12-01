#[phase(plugin)]
extern crate regex_macros;
extern crate regex;
extern crate url;

use super::util::{ReadUntilStr, HeaderCollection, ReadHttpHeaders, WriteHttpHeaders};
use super::version::HttpVersion;
use url::{Url, ParseResult, ParseError};
use std::fmt::Show;
use std::rand;
use std::io::{Reader, Writer, IoResult, IoError, IoErrorKind};
use serialize::base64::{ToBase64, STANDARD};
use std::clone::Clone;

/// Represents a WebSocket handshake request, which is sent from the client to the server.
/// Use the new() function to create a new request, and send it with the WebSocketClient::connect() function.
/// Use the WebSocketClient.receive_handshake_request() method to read a WebSocketRequest from a remote client.
/// 
/// ```
/// use websocket::handshake::WebSocketRequest;
///
/// let protocols = vec!["FirstPreference", "SecondPreference", "ThirdPreference"];
/// let request = WebSocketRequest::new("ws://www.example.com:1234", protocols.as_slice()).unwrap();
/// 
/// assert_eq!(request.host().unwrap().as_slice(), "www.example.com:1234");
/// assert_eq!(request.version().unwrap().as_slice(), "13");
/// //...
/// ```
pub struct WebSocketRequest {
	/// The resource name of the request. E.g. /path/to/resource for the URI ws://www.example.com/path/to/resource
	pub resource_name: String,
	
	/// The HTTP version of this request
	pub http_version: HttpVersion,
	
	/// The collection of headers contained in this request
	pub headers: HeaderCollection,
}

impl WebSocketRequest {
	/// Creates a new WebSocket handshake request for use with WebSocketClient::connect().
	/// The URI should use the ws:// scheme (wss:// not supported at this time).
	pub fn new<A: Show>(uri: &str, protocols: &[A]) -> ParseResult<WebSocketRequest> {
		let ws_uri = match Url::parse(uri) {
			Ok(uri) => { uri }
			Err(e) => { return Err(e); }
		};
		
		match ws_uri.scheme.as_slice() {
			"ws" => { }
			/* "wss" => {
				// Don't know how to handle secure WebSocket
			} */
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
		
		for protocol in protocols.iter() {
			headers.insert("Sec-WebSocket-Protocol", protocol);
		}
		
		Ok(WebSocketRequest {
			resource_name: resource_name,
			http_version: HttpVersion::new(1u8, Some(1u8)),
			headers: headers,
		})
	}
	
	/// Short-cut to get the Host field value of this request
	pub fn host(&self) -> Option<String> {
		self.headers.get("Host")
	}
	
	/// Short-cut to get the Connection field value of this request
	pub fn connection(&self) -> Option<Vec<String>> {
		match self.headers.get("Connection") {
			Some(connection) => {
				let mut result: Vec<String> = Vec::new();
				let mut values = connection.as_slice().split_str(",");
				for value in values {
					result.push(value.trim().to_string());
				}
				Some(result)
			}
			None => { None }
		}
	}
	
	/// Short-cut to get the Upgrade field value of this request
	pub fn upgrade(&self) -> Option<Vec<String>> {
		match self.headers.get("Upgrade") {
			Some(upgrade) => {
				let mut result: Vec<String> = Vec::new();
				let mut values = upgrade.as_slice().split_str(",");
				for value in values {
					result.push(value.trim().to_string());
				}
				Some(result)
			}
			None => { None }
		}
	}
	
	/// Short-cut to get the Sec-WebSocket-Version field value of this request
	pub fn version(&self) -> Option<String> {
		self.headers.get("Sec-WebSocket-Version")
	}
	
	/// Short-cut to get the Sec-WebSocket-Key field value of this request
	pub fn key(&self) -> Option<String> {
		self.headers.get("Sec-WebSocket-Key")
	}
	
	/// Short-cut to get the Sec-WebSocket-Protocol field value of this request
	pub fn protocol(&self) -> Option<Vec<String>> {
		match self.headers.get("Sec-WebSocket-Protocol") {
			Some(protocol) => {
				let mut result: Vec<String> = Vec::new();
				let mut values = protocol.as_slice().split_str(",");
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
	
	/// Short-cut to get the Origin field value of this request
	pub fn origin(&self) -> Option<String> {
		self.headers.get("Origin")
	}
}

impl Clone for WebSocketRequest {
	fn clone(&self) -> WebSocketRequest {
		WebSocketRequest {
			resource_name: self.resource_name.clone(),
			http_version: self.http_version,
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
		
		let re = regex!(r"GET (.*) HTTP/(\d+)(?:\.(\d+))?");
		let captures = re.captures(request_line.as_slice()).unwrap();
		
		let resource_name = captures.at(1);
		
		let version_major: Option<u8> = from_str(captures.at(2));
		let version_minor: Option<u8> = from_str(captures.at(3));
		
		let headers = try!(self.read_http_headers());
		
		Ok(WebSocketRequest {
			resource_name: resource_name.to_string(),
			http_version: HttpVersion::new(try!(version_major.ok_or(
				IoError {
					kind: IoErrorKind::InvalidInput,
					desc: "No HTTP version",
					detail: None,
				})
			), version_minor),
			headers: headers,
		})
	}
}

pub trait WriteWebSocketRequest {
	fn write_websocket_request(&mut self, request: &WebSocketRequest) -> IoResult<()>;
}

impl<W: Writer> WriteWebSocketRequest for W {
	fn write_websocket_request(&mut self, request: &WebSocketRequest) -> IoResult<()> {
		let request_line = "GET ".to_string() + request.resource_name + " HTTP/".to_string() + request.http_version.to_string();
		
		try!(self.write_str(request_line.as_slice()));
		try!(self.write_str("\r\n"));
		try!(self.write_http_headers(&(request.headers)));
		try!(self.write_str("\r\n"));
		
		Ok(())
	}
}