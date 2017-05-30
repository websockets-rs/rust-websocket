//! Allows you to take an existing request or stream of data and convert it into a
//! WebSocket client.
use std::error::Error;
use std::io;
use std::fmt::{self, Formatter, Display};
use stream::Stream;
use header::extensions::Extension;
use header::{WebSocketAccept, WebSocketKey, WebSocketVersion, WebSocketProtocol,
             WebSocketExtensions, Origin};

use unicase::UniCase;
use hyper::status::StatusCode;
use hyper::http::h1::Incoming;
use hyper::method::Method;
use hyper::uri::RequestUri;
use hyper::header::{Headers, Upgrade, Protocol, ProtocolName, Connection, ConnectionOption};

#[cfg(any(feature="sync", feature="async"))]
use hyper::version::HttpVersion;

#[cfg(feature="async")]
pub mod async;

#[cfg(feature="sync")]
pub mod sync;

/// A typical request from hyper
pub type Request = Incoming<(Method, RequestUri)>;

/// Intermediate representation of a half created websocket session.
/// Should be used to examine the client's handshake
/// accept the protocols requested, route the path, etc.
///
/// Users should then call `accept` or `reject` to complete the handshake
/// and start a session.
/// Note: if the stream in use is `AsyncRead + AsyncWrite`, then asynchronous
/// functions will be available when completing the handshake.
/// Otherwise if the stream is simply `Read + Write` blocking functions will be
/// available to complete the handshake.
pub struct WsUpgrade<S, B>
	where S: Stream
{
	/// The headers that will be used in the handshake response.
	pub headers: Headers,
	/// The stream that will be used to read from / write to.
	pub stream: S,
	/// The handshake request, filled with useful metadata.
	pub request: Request,
	/// Some buffered data from the stream, if it exists.
	pub buffer: B,
}

impl<S, B> WsUpgrade<S, B>
    where S: Stream
{
	/// Select a protocol to use in the handshake response.
	pub fn use_protocol<P>(mut self, protocol: P) -> Self
		where P: Into<String>
	{
		upsert_header!(self.headers; WebSocketProtocol; {
            Some(protos) => protos.0.push(protocol.into()),
            None => WebSocketProtocol(vec![protocol.into()])
        });
		self
	}

	/// Select an extension to use in the handshake response.
	pub fn use_extension(mut self, extension: Extension) -> Self {
		upsert_header!(self.headers; WebSocketExtensions; {
            Some(protos) => protos.0.push(extension),
            None => WebSocketExtensions(vec![extension])
        });
		self
	}

	/// Select multiple extensions to use in the connection
	pub fn use_extensions<I>(mut self, extensions: I) -> Self
		where I: IntoIterator<Item = Extension>
	{
		let mut extensions: Vec<Extension> =
			extensions.into_iter().collect();
		upsert_header!(self.headers; WebSocketExtensions; {
            Some(protos) => protos.0.append(&mut extensions),
            None => WebSocketExtensions(extensions)
        });
		self
	}

	/// Drop the connection without saying anything.
	pub fn drop(self) {
		::std::mem::drop(self);
	}

	/// A list of protocols requested from the client.
	pub fn protocols(&self) -> &[String] {
		self.request
		    .headers
		    .get::<WebSocketProtocol>()
		    .map(|p| p.0.as_slice())
		    .unwrap_or(&[])
	}

	/// A list of extensions requested from the client.
	pub fn extensions(&self) -> &[Extension] {
		self.request
		    .headers
		    .get::<WebSocketExtensions>()
		    .map(|e| e.0.as_slice())
		    .unwrap_or(&[])
	}

	/// The client's websocket accept key.
	pub fn key(&self) -> Option<&[u8; 16]> {
		self.request.headers.get::<WebSocketKey>().map(|k| &k.0)
	}

	/// The client's websocket version.
	pub fn version(&self) -> Option<&WebSocketVersion> {
		self.request.headers.get::<WebSocketVersion>()
	}

	/// Origin of the client
	pub fn origin(&self) -> Option<&str> {
		self.request.headers.get::<Origin>().map(|o| &o.0 as &str)
	}

	#[cfg(feature="sync")]
	fn send(&mut self, status: StatusCode) -> io::Result<()> {
		write!(&mut self.stream, "{} {}\r\n", self.request.version, status)?;
		write!(&mut self.stream, "{}\r\n", self.headers)?;
		Ok(())
	}

	#[doc(hidden)]
	pub fn prepare_headers(&mut self, custom: Option<&Headers>) -> StatusCode {
		if let Some(headers) = custom {
			self.headers.extend(headers.iter());
		}
		// NOTE: we know there is a key because this is a valid request
		// i.e. to construct this you must go through the validate function
		let key = self.request.headers.get::<WebSocketKey>().unwrap();
		self.headers.set(WebSocketAccept::new(key));
		self.headers
		    .set(Connection(vec![
			          ConnectionOption::ConnectionHeader(UniCase("Upgrade".to_string()))
		        ]));
		self.headers.set(Upgrade(vec![Protocol::new(ProtocolName::WebSocket, None)]));

		StatusCode::SwitchingProtocols
	}
}

/// Errors that can occur when one tries to upgrade a connection to a
/// websocket connection.
#[derive(Debug)]
pub enum HyperIntoWsError {
	/// The HTTP method in a valid websocket upgrade request must be GET
	MethodNotGet,
	/// Currently HTTP 2 is not supported
	UnsupportedHttpVersion,
	/// Currently only WebSocket13 is supported (RFC6455)
	UnsupportedWebsocketVersion,
	/// A websocket upgrade request must contain a key
	NoSecWsKeyHeader,
	/// A websocket upgrade request must ask to upgrade to a `websocket`
	NoWsUpgradeHeader,
	/// A websocket upgrade request must contain an `Upgrade` header
	NoUpgradeHeader,
	/// A websocket upgrade request's `Connection` header must be `Upgrade`
	NoWsConnectionHeader,
	/// A websocket upgrade request must contain a `Connection` header
	NoConnectionHeader,
	/// IO error from reading the underlying socket
	Io(io::Error),
	/// Error while parsing an incoming request
	Parsing(::hyper::error::Error),
}

impl Display for HyperIntoWsError {
	fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
		fmt.write_str(self.description())
	}
}

impl Error for HyperIntoWsError {
	fn description(&self) -> &str {
		use self::HyperIntoWsError::*;
		match *self {
			MethodNotGet => "Request method must be GET",
			UnsupportedHttpVersion => "Unsupported request HTTP version",
			UnsupportedWebsocketVersion => "Unsupported WebSocket version",
			NoSecWsKeyHeader => "Missing Sec-WebSocket-Key header",
			NoWsUpgradeHeader => "Invalid Upgrade WebSocket header",
			NoUpgradeHeader => "Missing Upgrade WebSocket header",
			NoWsConnectionHeader => "Invalid Connection WebSocket header",
			NoConnectionHeader => "Missing Connection WebSocket header",
			Io(ref e) => e.description(),
			Parsing(ref e) => e.description(),
		}
	}

	fn cause(&self) -> Option<&Error> {
		match *self {
			HyperIntoWsError::Io(ref e) => Some(e),
			HyperIntoWsError::Parsing(ref e) => Some(e),
			_ => None,
		}
	}
}

impl From<io::Error> for HyperIntoWsError {
	fn from(err: io::Error) -> Self {
		HyperIntoWsError::Io(err)
	}
}

impl From<::hyper::error::Error> for HyperIntoWsError {
	fn from(err: ::hyper::error::Error) -> Self {
		HyperIntoWsError::Parsing(err)
	}
}

#[cfg(feature="async")]
impl From<::codec::http::HttpCodecError> for HyperIntoWsError {
	fn from(src: ::codec::http::HttpCodecError) -> Self {
		match src {
			::codec::http::HttpCodecError::Io(e) => HyperIntoWsError::Io(e),
			::codec::http::HttpCodecError::Http(e) => HyperIntoWsError::Parsing(e),
		}
	}
}

#[cfg(any(feature="sync", feature="async"))]
fn validate(
	method: &Method,
	version: &HttpVersion,
	headers: &Headers,
) -> Result<(), HyperIntoWsError> {
	if *method != Method::Get {
		return Err(HyperIntoWsError::MethodNotGet);
	}

	if *version == HttpVersion::Http09 || *version == HttpVersion::Http10 {
		return Err(HyperIntoWsError::UnsupportedHttpVersion);
	}

	if let Some(version) = headers.get::<WebSocketVersion>() {
		if version != &WebSocketVersion::WebSocket13 {
			return Err(HyperIntoWsError::UnsupportedWebsocketVersion);
		}
	}

	if headers.get::<WebSocketKey>().is_none() {
		return Err(HyperIntoWsError::NoSecWsKeyHeader);
	}

	match headers.get() {
		Some(&Upgrade(ref upgrade)) => {
			if upgrade.iter().all(|u| u.name != ProtocolName::WebSocket) {
				return Err(HyperIntoWsError::NoWsUpgradeHeader);
			}
		}
		None => return Err(HyperIntoWsError::NoUpgradeHeader),
	};

	fn check_connection_header(headers: &[ConnectionOption]) -> bool {
		for header in headers {
			if let ConnectionOption::ConnectionHeader(ref h) = *header {
				if UniCase(h as &str) == UniCase("upgrade") {
					return true;
				}
			}
		}
		false
	}

	match headers.get() {
		Some(&Connection(ref connection)) => {
			if !check_connection_header(connection) {
				return Err(HyperIntoWsError::NoWsConnectionHeader);
			}
		}
		None => return Err(HyperIntoWsError::NoConnectionHeader),
	};

	Ok(())
}
