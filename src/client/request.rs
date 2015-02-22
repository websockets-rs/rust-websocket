//! Structs for client-side (outbound) WebSocket requests
pub use url::Url;

use hyper::version::HttpVersion;
use hyper::header::Headers;
use hyper::header::{Connection, ConnectionOption};
use hyper::header::{Upgrade, Protocol};

use unicase::UniCase;

use header::{WebSocketKey, WebSocketVersion, WebSocketProtocol, WebSocketExtensions, Origin};
use result::{WebSocketResult, WebSocketError};
use client::response::Response;
use ws::util::url::url_to_host;

/// Represents a WebSocket request.
///
/// Note that nothing is written to the internal Writer until the `send()` method is called.
#[derive(Debug)]
pub struct Request<R: Reader, W: Writer> {
	/// The target URI for this request.
    pub url: Url,
    /// The HTTP version of this request.
    pub version: HttpVersion,
	/// The headers of this request.
	pub headers: Headers,
	
	reader: R,
	writer: W,
}

unsafe impl<R, W> Send for Request<R, W> where R: Reader + Send, W: Writer + Send { }

impl<R: Reader, W: Writer> Request<R, W> {
	/// Creates a new client-side request.
	///
	/// In general `Client::connect()` should be used for connecting to servers.
	/// However, if the request is to be written to a different Writer, this function
	/// may be used.
	pub fn new(url: Url, reader: R, writer: W) -> WebSocketResult<Request<R, W>> {
		let mut headers = Headers::new();
		let host = try!(url_to_host(&url).ok_or(
			WebSocketError::RequestError("Could not get hostname and port from URL".to_string())
		));
		headers.set(host);
		headers.set(Connection(vec![
			ConnectionOption::ConnectionHeader(UniCase("Upgrade".to_string()))
		]));
		headers.set(Upgrade(vec![Protocol::WebSocket]));
		headers.set(WebSocketVersion::WebSocket13);
		headers.set(WebSocketKey::new());
		
		Ok(Request {
			url: url,
			version: HttpVersion::Http11,
			headers: headers,
			reader: reader,
			writer: writer
		})
	}
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
	/// Short-cut to obtain a mutable reference to the WebSocketKey value.
	///
	/// Note that to add a header that does not already exist, ```Request.headers.set()```
	/// must be used.
	pub fn key_mut(&mut self) -> Option<&mut WebSocketKey> {
		self.headers.get_mut()
	}
	/// Short-cut to obtain a mutable reference to the WebSocketVersion value.
	///
	/// Note that to add a header that does not already exist, ```Request.headers.set()```
	/// must be used.
	pub fn version_mut(&mut self) -> Option<&mut WebSocketVersion> {
		self.headers.get_mut()
	}
	/// Short-cut to obtaina mutable reference to  the WebSocketProtocol value.
	///
	/// Note that to add a header that does not already exist, ```Request.headers.set()```
	/// must be used.
	pub fn protocol_mut(&mut self) -> Option<&mut WebSocketProtocol> {
		self.headers.get_mut()
	}
	/// Short-cut to obtain a mutable reference to the WebSocketExtensions value.
	///
	/// Note that to add a header that does not already exist, ```Request.headers.set()```
	/// must be used.
	pub fn extensions_mut(&mut self) -> Option<&mut WebSocketExtensions> {
		self.headers.get_mut()
	}
	/// Short-cut to obtain a mutable reference to the Origin value.
	///
	/// Note that to add a header that does not already exist, ```Request.headers.set()```
	/// must be used.
	pub fn origin_mut(&mut self) -> Option<&mut Origin> {
		self.headers.get_mut()
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
	/// Return the inner Reader and Writer.
	pub fn into_inner(self) -> (R, W) {
		(self.reader, self.writer)
	}
	/// Sends the request to the server and returns a response.
	pub fn send(mut self) -> WebSocketResult<Response<R, W>> {
		let mut path = self.url.serialize_path().unwrap();
		match self.url.query.clone() {
			Some(query) => {
				path.push_str("?");
				path.push_str(&query[..]);
			}
			None => (),
		}
		try!(write!(&mut self.writer, "GET {} {}\r\n", path, self.version));
		try!(write!(&mut self.writer, "{}\r\n", self.headers));
		Response::read(self)
	}
}