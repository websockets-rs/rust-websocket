//! The server-side WebSocket request.

use std::io::{Read, Write};

use server::Response;
use result::{WebSocketResult, WebSocketError};
use header::{WebSocketKey, WebSocketVersion, WebSocketProtocol, WebSocketExtensions, Origin};

pub use hyper::uri::RequestUri;

use hyper::buffer::BufReader;
use hyper::version::HttpVersion;
use hyper::header::Headers;
use hyper::header::{Connection, ConnectionOption};
use hyper::header::{Upgrade, ProtocolName};
use hyper::http::h1::parse_request;
use hyper::method::Method;

use unicase::UniCase;

/// Represents a server-side (incoming) request.
pub struct Request<R: Read, W: Write> {
	/// The target URI for this request.
    pub url: RequestUri,
	
    /// The HTTP version of this request.
    pub version: HttpVersion,
	
	/// The headers of this request.
	pub headers: Headers,
	
	reader: R,
	writer: W,
}

unsafe impl<R, W> Send for Request<R, W> where R: Read + Send, W: Write + Send { }

impl<R: Read, W: Write> Request<R, W> {
	/// Short-cut to obtain the WebSocketKey value.
	pub fn key(&self) -> Option<&WebSocketKey> {
		self.headers.get()
	}
	/// Short-cut to obtain the WebSocketVersion value.
	pub fn version(&self) -> Option<&WebSocketVersion> {
		self.headers.get()
	}
	/// Short-cut to obtain the WebSocketProtocol value.
	pub fn protocol(&self) -> Option<&WebSocketProtocol> {
		self.headers.get()
	}
	/// Short-cut to obtain the WebSocketExtensions value.
	pub fn extensions(&self) -> Option<&WebSocketExtensions> {
		self.headers.get()
	}
	/// Short-cut to obtain the Origin value.
	pub fn origin(&self) -> Option<&Origin> {
		self.headers.get()
	}
	/// Returns a reference to the inner Reader.
	pub fn get_reader(&self) -> &R {
		&self.reader
	}
	/// Returns a reference to the inner Writer.
	pub fn get_writer(&self) -> &W {
		&self.writer
	}
	/// Returns a mutable reference to the inner Reader.
	pub fn get_mut_reader(&mut self) -> &mut R {
		&mut self.reader
	}
	/// Returns a mutable reference to the inner Writer.
	pub fn get_mut_writer(&mut self) -> &mut W {
		&mut self.writer
	}
	/// Return the inner Reader and Writer
	pub fn into_inner(self) -> (R, W) {
		(self.reader, self.writer)
	}
	/// Reads an inbound request.
	/// 
	/// This method is used within servers, and returns an inbound WebSocketRequest.
	/// An error will be returned if the request cannot be read, or is not a valid HTTP request.
	pub fn read(reader: R, writer: W) -> WebSocketResult<Request<R, W>> {
		let mut reader = BufReader::new(reader);
		let request = try!(parse_request(&mut reader));
		
		match request.subject.0 {
			Method::Get => { },
			_ => { return Err(WebSocketError::RequestError("Request method must be GET".to_string())); }
		}
		
		Ok(Request {
			url: request.subject.1,
			version: request.version,
			headers: request.headers,
			reader: reader.into_inner(),
			writer: writer,
		})
	}
	/// Check if this constitutes a valid request.
	///
	/// Note that `accept()` calls this function internally, however this may be useful for handling bad requests
	/// in a custom way.
	pub fn validate(&self) -> WebSocketResult<()> {
		if self.version == HttpVersion::Http09 || self.version == HttpVersion::Http10 {
			return Err(WebSocketError::RequestError("Unsupported request HTTP version".to_string()));
		}
		
		if self.version() != Some(&(WebSocketVersion::WebSocket13)) {
			return Err(WebSocketError::RequestError("Unsupported WebSocket version".to_string()));
		}
		
		if self.key().is_none() {
			return Err(WebSocketError::RequestError("Missing Sec-WebSocket-Key header".to_string()));
		}
		
		match self.headers.get() {
			Some(&Upgrade(ref upgrade)) => {
				let mut correct_upgrade = false;
				for u in upgrade {
					if u.name == ProtocolName::WebSocket {
						correct_upgrade = true;
					}
				}
				if !correct_upgrade {
					return Err(WebSocketError::RequestError("Invalid Upgrade WebSocket header".to_string()));
				}
			}
			None => { return Err(WebSocketError::RequestError("Missing Upgrade WebSocket header".to_string())); }
		}
		
		match self.headers.get() {
			Some(&Connection(ref connection)) => {
				if !connection.contains(&(ConnectionOption::ConnectionHeader(UniCase("Upgrade".to_string())))) {
					return Err(WebSocketError::RequestError("Invalid Connection WebSocket header".to_string()));
				}
			}
			None => { return Err(WebSocketError::RequestError("Missing Connection WebSocket header".to_string())); }
		}
		
		Ok(())
	}
	
	/// Accept this request, ready to send a response.
	///
	/// This function calls `validate()` on the request, and if the request is found to be invalid,
	/// generates a response with a Bad Request status code.
	pub fn accept(self) -> Response<R, W> {
		match self.validate() {
			Ok(()) => { }
			Err(_) => { return self.fail(); }
		}
		Response::new(self)
	}
	
	/// Fail this request by generating a Bad Request response
	pub fn fail(self) -> Response<R, W> {
		Response::bad_request(self)
	}
}
