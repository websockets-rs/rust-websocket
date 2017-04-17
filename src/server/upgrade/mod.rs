//! Allows you to take an existing request or stream of data and convert it into a
//! WebSocket client.
use std::error::Error;
use std::net::TcpStream;
use std::io;
use std::io::Result as IoResult;
use std::io::Error as IoError;
use std::fmt::{self, Formatter, Display};
use stream::{Stream, AsTcpStream};
use header::extensions::Extension;
use header::{WebSocketAccept, WebSocketKey, WebSocketVersion, WebSocketProtocol,
             WebSocketExtensions, Origin};
use client::Client;

use unicase::UniCase;
use hyper::status::StatusCode;
use hyper::http::h1::Incoming;
use hyper::method::Method;
use hyper::version::HttpVersion;
use hyper::uri::RequestUri;
use hyper::buffer::BufReader;
use hyper::http::h1::parse_request;
use hyper::header::{Headers, Upgrade, Protocol, ProtocolName, Connection, ConnectionOption};

pub mod from_hyper;

/// This crate uses buffered readers to read in the handshake quickly, in order to
/// interface with other use cases that don't use buffered readers the buffered readers
/// is deconstructed when it is returned to the user and given as the underlying
/// reader and the buffer.
///
/// This struct represents bytes that have already been read in from the stream.
/// A slice of valid data in this buffer can be obtained by: `&buf[pos..cap]`.
#[derive(Debug)]
pub struct Buffer {
	/// the contents of the buffered stream data
	pub buf: Vec<u8>,
	/// the current position of cursor in the buffer
	/// Any data before `pos` has already been read and parsed.
	pub pos: usize,
	/// the last location of valid data
	/// Any data after `cap` is not valid.
	pub cap: usize,
}

/// Intermediate representation of a half created websocket session.
/// Should be used to examine the client's handshake
/// accept the protocols requested, route the path, etc.
///
/// Users should then call `accept` or `reject` to complete the handshake
/// and start a session.
pub struct WsUpgrade<S>
	where S: Stream
{
	/// The headers that will be used in the handshake response.
	pub headers: Headers,
	/// The stream that will be used to read from / write to.
	pub stream: S,
	/// The handshake request, filled with useful metadata.
	pub request: Request,
	/// Some buffered data from the stream, if it exists.
	pub buffer: Option<Buffer>,
}

impl<S> WsUpgrade<S>
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

	/// Accept the handshake request and send a response,
	/// if nothing goes wrong a client will be created.
	pub fn accept(self) -> Result<Client<S>, (S, IoError)> {
		self.accept_with(&Headers::new())
	}

	/// Accept the handshake request and send a response while
	/// adding on a few headers. These headers are added before the required
	/// headers are, so some might be overwritten.
	pub fn accept_with(mut self, custom_headers: &Headers) -> Result<Client<S>, (S, IoError)> {
		self.headers.extend(custom_headers.iter());
		self.headers
		    .set(WebSocketAccept::new(// NOTE: we know there is a key because this is a valid request
		                              // i.e. to construct this you must go through the validate function
		                              self.request.headers.get::<WebSocketKey>().unwrap()));
		self.headers
		    .set(Connection(vec![
			      ConnectionOption::ConnectionHeader(UniCase("Upgrade".to_string()))
		    ]));
		self.headers.set(Upgrade(vec![Protocol::new(ProtocolName::WebSocket, None)]));

		if let Err(e) = self.send(StatusCode::SwitchingProtocols) {
			return Err((self.stream, e));
		}

		let stream = match self.buffer {
			Some(Buffer { buf, pos, cap }) => BufReader::from_parts(self.stream, buf, pos, cap),
			None => BufReader::new(self.stream),
		};

		Ok(Client::unchecked(stream, self.headers, false, true))
	}

	/// Reject the client's request to make a websocket connection.
	pub fn reject(self) -> Result<S, (S, IoError)> {
		self.reject_with(&Headers::new())
	}
	/// Reject the client's request to make a websocket connection
	/// and send extra headers.
	pub fn reject_with(mut self, headers: &Headers) -> Result<S, (S, IoError)> {
		self.headers.extend(headers.iter());
		match self.send(StatusCode::BadRequest) {
			Ok(()) => Ok(self.stream),
			Err(e) => Err((self.stream, e)),
		}
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

	fn send(&mut self, status: StatusCode) -> IoResult<()> {
		try!(write!(&mut self.stream, "{} {}\r\n", self.request.version, status));
		try!(write!(&mut self.stream, "{}\r\n", self.headers));
		Ok(())
	}
}

impl<S> WsUpgrade<S>
    where S: Stream + AsTcpStream
{
	/// Get a handle to the underlying TCP stream, useful to be able to set
	/// TCP options, etc.
	pub fn tcp_stream(&self) -> &TcpStream {
		self.stream.as_tcp()
	}
}

/// Trait to take a stream or similar and attempt to recover the start of a
/// websocket handshake from it.
/// Should be used when a stream might contain a request for a websocket session.
///
/// If an upgrade request can be parsed, one can accept or deny the handshake with
/// the `WsUpgrade` struct.
/// Otherwise the original stream is returned along with an error.
///
/// Note: the stream is owned because the websocket client expects to own its stream.
///
/// This is already implemented for all Streams, which means all types with Read + Write.
///
/// # Example
///
/// ```rust,no_run
/// use std::net::TcpListener;
/// use std::net::TcpStream;
/// use websocket::server::upgrade::IntoWs;
/// use websocket::Client;
///
/// let listener = TcpListener::bind("127.0.0.1:80").unwrap();
///
/// for stream in listener.incoming().filter_map(Result::ok) {
///     let mut client: Client<TcpStream> = match stream.into_ws() {
/// 		    Ok(upgrade) => {
///             match upgrade.accept() {
///                 Ok(client) => client,
///                 Err(_) => panic!(),
///             }
///         },
/// 		    Err(_) => panic!(),
///     };
/// }
/// ```
pub trait IntoWs {
	/// The type of stream this upgrade process is working with (TcpStream, etc.)
	type Stream: Stream;
	/// An error value in case the stream is not asking for a websocket connection
	/// or something went wrong. It is common to also include the stream here.
	type Error;
	/// Attempt to parse the start of a Websocket handshake, later with the  returned
	/// `WsUpgrade` struct, call `accept to start a websocket client, and `reject` to
	/// send a handshake rejection response.
	fn into_ws(self) -> Result<WsUpgrade<Self::Stream>, Self::Error>;
}


/// A typical request from hyper
pub type Request = Incoming<(Method, RequestUri)>;
/// If you have your requests separate from your stream you can use this struct
/// to upgrade the connection based on the request given
/// (the request should be a handshake).
pub struct RequestStreamPair<S: Stream>(pub S, pub Request);

impl<S> IntoWs for S
    where S: Stream
{
	type Stream = S;
	type Error = (S, Option<Request>, Option<Buffer>, HyperIntoWsError);

	fn into_ws(self) -> Result<WsUpgrade<Self::Stream>, Self::Error> {
		let mut reader = BufReader::new(self);
		let request = parse_request(&mut reader);

		let (stream, buf, pos, cap) = reader.into_parts();
		let buffer = Some(Buffer {
		                      buf: buf,
		                      cap: cap,
		                      pos: pos,
		                  });

		let request = match request {
			Ok(r) => r,
			Err(e) => return Err((stream, None, buffer, e.into())),
		};

		match validate(&request.subject.0, &request.version, &request.headers) {
			Ok(_) => {
				Ok(WsUpgrade {
				       headers: Headers::new(),
				       stream: stream,
				       request: request,
				       buffer: buffer,
				   })
			}
			Err(e) => Err((stream, Some(request), buffer, e)),
		}
	}
}

impl<S> IntoWs for RequestStreamPair<S>
    where S: Stream
{
	type Stream = S;
	type Error = (S, Request, HyperIntoWsError);

	fn into_ws(self) -> Result<WsUpgrade<Self::Stream>, Self::Error> {
		match validate(&self.1.subject.0, &self.1.version, &self.1.headers) {
			Ok(_) => {
				Ok(WsUpgrade {
				       headers: Headers::new(),
				       stream: self.0,
				       request: self.1,
				       buffer: None,
				   })
			}
			Err(e) => Err((self.0, self.1, e)),
		}
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

	fn check_connection_header(headers: &Vec<ConnectionOption>) -> bool {
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
