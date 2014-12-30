#![unstable]
//! Structs for WebSocket requests
use std::io::TcpStream;
use std::io::net::ip::Port;
pub use url::Url;
pub use hyper::uri::RequestUri;
use hyper::version::HttpVersion;
use hyper::status::StatusCode;
use hyper::header::Headers;
use hyper::header::common::{Host, Connection, Upgrade};
use hyper::header::connection::ConnectionOption;
use hyper::header::common::upgrade::Protocol;
use hyper::http::read_request_line;
use hyper::method::Method;
use header::{WebSocketKey, WebSocketAccept, WebSocketVersion, WebSocketProtocol, WebSocketExtensions, Origin};
use handshake::response::WebSocketResponse;
use common::{Inbound, Outbound, WebSocketStream, WebSocketResult, WebSocketError};
use openssl::ssl::{SslMethod, SslContext, SslStream};

/// Represents a WebSocket request
/// 
/// A WebSocketRequest is used to either make a connection to a WebSocket server using ```connect()```
/// or ```connect_with()```, or received by a server and processed using ```read()``` and ```accept()```.
/// 
/// A type parameter that is either Inbound or Outbound ensures that only the appropriate methods
/// can be called on either kind of request.
/// 
/// The headers field allows access to the HTTP headers in the request, but short-cut methods are
/// available for accessing common headers.
pub struct WebSocketRequest<R: Reader, W: Writer, B> {
	/// The target URI for this request.
    pub url: RequestUri,
	
    /// The HTTP version of this request.
    pub version: HttpVersion,
	
	/// The headers of this request.
	pub headers: Headers,
	
	reader: R,
	writer: W,
}

impl<R: Reader + Send, W: Writer + Send, B> WebSocketRequest<R, W, B> {
	/// Short-cut to obtain the WebSocketKey value
	pub fn key(&self) -> Option<&WebSocketKey> {
		self.headers.get()
	}
	/// Short-cut to obtain the WebSocketVersion value
	pub fn version(&self) -> Option<&WebSocketVersion> {
		self.headers.get()
	}
	/// Short-cut to obtain the WebSocketProtocol value
	pub fn protocol(&self) -> Option<&WebSocketProtocol> {
		self.headers.get()
	}
	/// Short-cut to obtain the WebSocketExtensions value
	pub fn extensions(&self) -> Option<&WebSocketExtensions> {
		self.headers.get()
	}
	/// Short-cut to obtain the Origin value
	pub fn origin(&self) -> Option<&Origin> {
		self.headers.get()
	}
}

impl WebSocketRequest<WebSocketStream, WebSocketStream, Outbound> {
	/// Connects to the specified ws:// or wss:// URL.
	///
	/// If a wss:// URL is supplied, a default SslContext is used.
	/// ```connect_ssl_context()``` can be used to supply an SslContext
	/// for use.
	pub fn connect(url: Url) -> WebSocketResult<WebSocketRequest<WebSocketStream, WebSocketStream, Outbound>> {
		let (host, port) = match get_host_and_port(&url) {
			Some((host, port)) => (host, port),
			None => { return Err(WebSocketError::RequestError("Could not get host and port for connection".to_string())); }
		};
		
		let stream = try!(TcpStream::connect((host.clone() + ":" +  port.to_string().as_slice()).as_slice()));
		
		let httpstream = match url.scheme.as_slice() {
			"ws" => {
				WebSocketStream::Normal(stream)
			}
			"wss" => {
				let context = SslContext::new(SslMethod::Tlsv1).unwrap();
				let sslstream = SslStream::new(&context, stream).unwrap();
				WebSocketStream::Secure(sslstream)
			}
			_ => { return Err(WebSocketError::RequestError("URI scheme not supported".to_string())); }
		};
		
		WebSocketRequest::new_with(host, port, url, httpstream.clone(), httpstream.clone())
	}
	
	/// Connects to the specified wss:// URL using the given SSL context
	pub fn connect_ssl_context(url: Url, context: &SslContext) -> WebSocketResult<WebSocketRequest<WebSocketStream, WebSocketStream, Outbound>> {
		let (host, port) = match get_host_and_port(&url) {
			Some((host, port)) => (host, port),
			None => { return Err(WebSocketError::RequestError("Could not get host and port for connection".to_string())); }
		};
		
		let stream = try!(TcpStream::connect((host.clone() + ":" +  port.to_string().as_slice()).as_slice()));
		
		let httpstream = match url.scheme.as_slice() {
			"wss" => {
				let sslstream = SslStream::new(context, stream).unwrap();
				WebSocketStream::Secure(sslstream)
			}
			_ => { return Err(WebSocketError::RequestError("URI scheme not supported".to_string())); }
		};
		
		WebSocketRequest::new_with(host, port, url, httpstream.clone(), httpstream.clone())
	}
}

impl<R: Reader + Send, W: Writer + Send> WebSocketRequest<R, W, Outbound> {
	/// Create a new outbound request using the specified Reader and Writer.
	pub fn new(url: Url, reader: R, writer: W) -> WebSocketResult<WebSocketRequest<R, W, Outbound>> {
		let (host, port) = match get_host_and_port(&url) {
			Some((host, port)) => (host, port),
			None => { return Err(WebSocketError::RequestError("Could not get host and port for connection".to_string())); }
		};
		WebSocketRequest::new_with(host.clone(), port, url, reader, writer)
	}
	
	fn new_with(host: String, port: Port, url: Url, reader: R, writer: W) -> WebSocketResult<WebSocketRequest<R, W, Outbound>> {
		let mut headers = Headers::new();
		
		headers.set(Host {
			hostname: host,
			port: Some(port),
		});
		
		headers.set(Connection(vec![
			ConnectionOption::ConnectionHeader("Upgrade".to_string())
		]));
		
		headers.set(Upgrade(vec![Protocol::WebSocket]));
		headers.set(WebSocketVersion::WebSocket13);
		headers.set(WebSocketKey::new());
		
		Ok(WebSocketRequest {
			url: RequestUri::AbsoluteUri(url),
			version: HttpVersion::Http11,
			headers: headers,
			reader: reader,
			writer: writer,
		})
	}
	
	/// Short-cut to obtain a mutable reference to the WebSocketKey value
	/// Note that to add a header that does not already exist, ```WebSocketRequest.headers.set()```
	/// must be used.
	pub fn key_mut(&mut self) -> Option<&mut WebSocketKey> {
		self.headers.get_mut()
	}
	/// Short-cut to obtain a mutable reference to the WebSocketVersion value
	/// Note that to add a header that does not already exist, ```WebSocketRequest.headers.set()```
	/// must be used.
	pub fn version_mut(&mut self) -> Option<&mut WebSocketVersion> {
		self.headers.get_mut()
	}
	/// Short-cut to obtaina mutable reference to  the WebSocketProtocol value
	/// Note that to add a header that does not already exist, ```WebSocketRequest.headers.set()```
	/// must be used.
	pub fn protocol_mut(&mut self) -> Option<&mut WebSocketProtocol> {
		self.headers.get_mut()
	}
	/// Short-cut to obtain a mutable reference to the WebSocketExtensions value
	/// Note that to add a header that does not already exist, ```WebSocketRequest.headers.set()```
	/// must be used.
	pub fn extensions_mut(&mut self) -> Option<&mut WebSocketExtensions> {
		self.headers.get_mut()
	}
	/// Short-cut to obtain a mutable reference to the Origin value
	/// Note that to add a header that does not already exist, ```WebSocketRequest.headers.set()```
	/// must be used.
	pub fn origin_mut(&mut self) -> Option<&mut Origin> {
		self.headers.get_mut()
	}
	
	/// Sends the request to the server and returns a response.
	pub fn send(self) -> WebSocketResult<WebSocketResponse<R, W, Inbound>> {
		let uri = match self.url {
			RequestUri::AbsoluteUri(url) => { url.serialize_path().unwrap() }
			_ =>  { return Err(WebSocketError::RequestError("An absolute URI must be used to make a connection".to_string())); }
		};
		let mut writer = self.writer;
		try!(write!(&mut writer, "GET {} {}\r\n", uri, self.version));
		try!(write!(&mut writer, "{}\r\n", self.headers));
		WebSocketResponse::read(self.reader, writer)
	}
}

impl<R: Reader + Send, W: Writer + Send> WebSocketRequest<R, W, Inbound> {
	/// Reads an inbound request.
	/// 
	/// This method is used within servers, and returns an inbound WebSocketRequest.
	/// An error will be returned if the request read does not constitute a 
	pub fn read(reader: R, writer: W) -> WebSocketResult<WebSocketRequest<R, W, Inbound>> {
		let mut reader = reader;
		let (method, uri, version) = try!(read_request_line(&mut reader));
		
		match method {
			Method::Get => { },
			_ => { return Err(WebSocketError::RequestError("WebSocketRequest method must be GET".to_string())); }
		}
		
        let headers = try!(Headers::from_raw(&mut reader));
		
		Ok(WebSocketRequest {
			url: uri,
			version: version,
			headers: headers,
			reader: reader,
			writer: writer,
		})
	}
	/// Check if this constitutes a valid request.
	///
	/// Note that ```accept()``` calls this function internally, however this may be useful for handling bad requests
	/// in a custom way.
	pub fn validate(&self) -> WebSocketResult<()> {
		if self.version == HttpVersion::Http09 || self.version == HttpVersion::Http10 {
			return Err(WebSocketError::RequestError("Unsupported WebSocketRequest HTTP version".to_string()));
		}
		
		if self.version() != Some(&(WebSocketVersion::WebSocket13)) {
			return Err(WebSocketError::RequestError("Unsupported WebSocket version".to_string()));
		}
		
		if self.key().is_none() {
			return Err(WebSocketError::RequestError("Missing Sec-WebSocket-Key header".to_string()));
		}
		
		match self.headers.get() {
			Some(&Upgrade(ref upgrade)) => {
				if !upgrade.contains(&(Protocol::WebSocket)) {
					return Err(WebSocketError::RequestError("Invalid Upgrade WebSocket header".to_string()));
				}
			}
			None => { return Err(WebSocketError::RequestError("Missing Upgrade WebSocket header".to_string())); }
		}
		
		match self.headers.get() {
			Some(&Connection(ref connection)) => {
				if !connection.contains(&(ConnectionOption::ConnectionHeader("Upgrade".to_string()))) {
					return Err(WebSocketError::RequestError("Invalid Connection WebSocket header".to_string()));
				}
			}
			None => { return Err(WebSocketError::RequestError("Missing Connection WebSocket header".to_string())); }
		}
		
		Ok(())
	}
	
	/// Accept this request, ready to send a response.
	///
	/// This function calls ```validate()``` on the request, and if the request is found to be invalid,
	/// generates a response with a Bad Request status code.
	pub fn accept(self) -> WebSocketResponse<R, W, Outbound> {
		match self.validate() {
			Ok(()) => { }
			Err(_) => { return self.fail(); }
		}
		let mut headers = Headers::new();
		headers.set(WebSocketAccept::new(self.key().unwrap()));
		headers.set(Connection(vec![
			ConnectionOption::ConnectionHeader("Upgrade".to_string())
		]));
		headers.set(Upgrade(vec![Protocol::WebSocket]));
		WebSocketResponse::new(StatusCode::SwitchingProtocols, headers, HttpVersion::Http11, self.reader, self.writer)
	}
	
	/// Fail this request by generating a Bad Request response
	pub fn fail(self) -> WebSocketResponse<R, W, Outbound> {
		WebSocketResponse::new(StatusCode::BadRequest, Headers::new(), HttpVersion::Http11, self.reader, self.writer)
	}
	
	/// Return the inner Reader and Writer
	pub fn into_inner(self) -> (R, W) {
		(self.reader, self.writer)
	}
}

fn get_host_and_port(url: &Url) -> Option<(String, Port)> {
    let host = match url.serialize_host() {
        Some(host) => host,
        None => { return None; }
    };
    let port = match url.port_or_default() {
        Some(port) => port,
        None => { return None; }
    };
    Some((host, port))
}