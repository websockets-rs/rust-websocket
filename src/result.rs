//! The result type used within Rust-WebSocket

use crate::server::upgrade::HyperIntoWsError;
pub use hyper::status::StatusCode;
use hyper::Error as HttpError;
use std::convert::From;
use std::error::Error;
use std::fmt;
use std::io;

use url::ParseError;

/// The type used for WebSocket results
pub type WebSocketResult<T> = Result<T, WebSocketError>;


pub use websocket_base::result::WebSocketError;

/// Represents a WebSocket error while connecting
#[derive(Debug)]
pub enum WebSocketOtherError {
	/// A WebSocket protocol error
	ProtocolError(&'static str),
	/// Invalid WebSocket request error
	RequestError(&'static str),
	/// Invalid WebSocket response error
	ResponseError(&'static str),
	/// Received unexpected status code
	StatusCodeError(StatusCode),
	/// An HTTP parsing error
	HttpError(HttpError),
	/// A URL parsing error
	UrlError(ParseError),
	/// An input/output error
	IoError(io::Error),
	/// A WebSocket URL error
	WebSocketUrlError(WSUrlErrorKind),
}

impl fmt::Display for WebSocketOtherError {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		match self {
			WebSocketOtherError::RequestError(e) => write!(fmt, "WebSocket request error: {}", e)?,
			WebSocketOtherError::ResponseError(e) => {
				write!(fmt, "WebSocket response error: {}", e)?
			}
			WebSocketOtherError::StatusCodeError(e) => write!(
				fmt,
				"WebSocketError: Received unexpected status code ({})",
				e
			)?,
			WebSocketOtherError::HttpError(e) => write!(fmt, "WebSocket HTTP error: {}", e)?,
			WebSocketOtherError::UrlError(e) => write!(fmt, "WebSocket URL parse error: {}", e)?,
			WebSocketOtherError::IoError(e) => write!(fmt, "WebSocket I/O error: {}", e)?,
			WebSocketOtherError::WebSocketUrlError(e) => e.fmt(fmt)?,
			_ => write!(fmt, "WebSocketError: {}", self.description())?,
		}
		Ok(())
	}
}

impl Error for WebSocketOtherError {
	fn description(&self) -> &str {
		match *self {
			WebSocketOtherError::RequestError(_) => "WebSocket request error",
			WebSocketOtherError::ResponseError(_) => "WebSocket response error",
			WebSocketOtherError::HttpError(_) => "HTTP failure",
			WebSocketOtherError::UrlError(_) => "URL failure",
			WebSocketOtherError::WebSocketUrlError(_) => "WebSocket URL failure",
			WebSocketOtherError::IoError(ref e) => e.description(),
			WebSocketOtherError::ProtocolError(e) => e,
			WebSocketOtherError::StatusCodeError(_) => "Received unexpected status code",
		}
	}

	fn cause(&self) -> Option<&dyn Error> {
		match *self {
			WebSocketOtherError::HttpError(ref error) => Some(error),
			WebSocketOtherError::UrlError(ref error) => Some(error),
			WebSocketOtherError::WebSocketUrlError(ref error) => Some(error),
			WebSocketOtherError::IoError(ref e) => Some(e),
			_ => None,
		}
	}
}

impl From<HttpError> for WebSocketOtherError {
	fn from(err: HttpError) -> WebSocketOtherError {
		WebSocketOtherError::HttpError(err)
	}
}

impl From<ParseError> for WebSocketOtherError {
	fn from(err: ParseError) -> WebSocketOtherError {
		WebSocketOtherError::UrlError(err)
	}
}

impl From<WSUrlErrorKind> for WebSocketOtherError {
	fn from(err: WSUrlErrorKind) -> WebSocketOtherError {
		WebSocketOtherError::WebSocketUrlError(err)
	}
}

impl From<HyperIntoWsError> for WebSocketOtherError {
	fn from(err: HyperIntoWsError) -> WebSocketOtherError {
		use self::HyperIntoWsError::*;
		use self::WebSocketOtherError::*;
		match err {
			Io(io) => IoError(io),
			Parsing(err) => HttpError(err),
			MethodNotGet => ProtocolError("Request method must be GET"),
			UnsupportedHttpVersion => ProtocolError("Unsupported request HTTP version"),
			UnsupportedWebsocketVersion => ProtocolError("Unsupported WebSocket version"),
			NoSecWsKeyHeader => ProtocolError("Missing Sec-WebSocket-Key header"),
			NoWsUpgradeHeader => ProtocolError("Invalid Upgrade WebSocket header"),
			NoUpgradeHeader => ProtocolError("Missing Upgrade WebSocket header"),
			NoWsConnectionHeader => ProtocolError("Invalid Connection WebSocket header"),
			NoConnectionHeader => ProtocolError("Missing Connection WebSocket header"),
		}
	}
}

/// Represents a WebSocket URL error
#[derive(Debug)]
pub enum WSUrlErrorKind {
	/// Fragments are not valid in a WebSocket URL
	CannotSetFragment,
	/// The scheme provided is invalid for a WebSocket
	InvalidScheme,
	/// There is no hostname or IP address to connect to
	NoHostName,
}

impl fmt::Display for WSUrlErrorKind {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.write_str("WebSocket Url Error: ")?;
		fmt.write_str(self.description())?;
		Ok(())
	}
}

impl Error for WSUrlErrorKind {
	fn description(&self) -> &str {
		match *self {
			WSUrlErrorKind::CannotSetFragment => "WebSocket URL cannot set fragment",
			WSUrlErrorKind::InvalidScheme => "WebSocket URL invalid scheme",
			WSUrlErrorKind::NoHostName => "WebSocket URL no host name provided",
		}
	}
}

impl From<WebSocketOtherError> for WebSocketError {
	fn from(e: WebSocketOtherError) -> WebSocketError {
		WebSocketError::Other(Box::new(e))
	}
}

pub(crate) fn towse<E>(e: E) -> WebSocketError
where
	E: Into<WebSocketOtherError>,
{
	let e: WebSocketOtherError = e.into();
	e.into()
}
