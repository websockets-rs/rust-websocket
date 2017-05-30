//! The result type used within Rust-WebSocket

use std::io;
use std::str::Utf8Error;
use std::error::Error;
use std::convert::From;
use std::fmt;
use hyper::Error as HttpError;
use url::ParseError;
use server::upgrade::HyperIntoWsError;

#[cfg(any(feature="sync-ssl", feature="async-ssl"))]
use native_tls::Error as TlsError;
#[cfg(any(feature="sync-ssl", feature="async-ssl"))]
use native_tls::HandshakeError as TlsHandshakeError;

/// The type used for WebSocket results
pub type WebSocketResult<T> = Result<T, WebSocketError>;

/// This module contains convenience types to make working with Futures and
/// websocket results easier.
#[cfg(feature="async")]
pub mod async {
	use futures::Future;
	use super::WebSocketError;

	/// The most common Future in this library, it is simply some result `I` or
	/// a `WebSocketError`. This is analogous to the `WebSocketResult` type.
	pub type WebSocketFuture<I> = Box<Future<Item = I, Error = WebSocketError>>;
}

/// Represents a WebSocket error
#[derive(Debug)]
pub enum WebSocketError {
	/// A WebSocket protocol error
	ProtocolError(&'static str),
	/// Invalid WebSocket request error
	RequestError(&'static str),
	/// Invalid WebSocket response error
	ResponseError(&'static str),
	/// Invalid WebSocket data frame error
	DataFrameError(&'static str),
	/// No data available
	NoDataAvailable,
	/// An input/output error
	IoError(io::Error),
	/// An HTTP parsing error
	HttpError(HttpError),
	/// A URL parsing error
	UrlError(ParseError),
	/// A WebSocket URL error
	WebSocketUrlError(WSUrlErrorKind),
	/// An SSL error
	#[cfg(any(feature="sync-ssl", feature="async-ssl"))]
	TlsError(TlsError),
	/// an ssl handshake failure
	#[cfg(any(feature="sync-ssl", feature="async-ssl"))]
	TlsHandshakeFailure,
	/// an ssl handshake interruption
	#[cfg(any(feature="sync-ssl", feature="async-ssl"))]
	TlsHandshakeInterruption,
	/// A UTF-8 error
	Utf8Error(Utf8Error),
}

impl fmt::Display for WebSocketError {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.write_str("WebSocketError: ")?;
		fmt.write_str(self.description())?;
		Ok(())
	}
}

impl Error for WebSocketError {
	fn description(&self) -> &str {
		match *self {
			WebSocketError::ProtocolError(_) => "WebSocket protocol error",
			WebSocketError::RequestError(_) => "WebSocket request error",
			WebSocketError::ResponseError(_) => "WebSocket response error",
			WebSocketError::DataFrameError(_) => "WebSocket data frame error",
			WebSocketError::NoDataAvailable => "No data available",
			WebSocketError::IoError(_) => "I/O failure",
			WebSocketError::HttpError(_) => "HTTP failure",
			WebSocketError::UrlError(_) => "URL failure",
			#[cfg(any(feature="sync-ssl", feature="async-ssl"))]
			      WebSocketError::TlsError(_) => "TLS failure",
			#[cfg(any(feature="sync-ssl", feature="async-ssl"))]
            WebSocketError::TlsHandshakeFailure => "TLS Handshake failure",
			#[cfg(any(feature="sync-ssl", feature="async-ssl"))]
            WebSocketError::TlsHandshakeInterruption => "TLS Handshake interrupted",
			WebSocketError::Utf8Error(_) => "UTF-8 failure",
			WebSocketError::WebSocketUrlError(_) => "WebSocket URL failure",
		}
	}

	fn cause(&self) -> Option<&Error> {
		match *self {
			WebSocketError::IoError(ref error) => Some(error),
			WebSocketError::HttpError(ref error) => Some(error),
			WebSocketError::UrlError(ref error) => Some(error),
			#[cfg(any(feature="sync-ssl", feature="async-ssl"))]
			      WebSocketError::TlsError(ref error) => Some(error),
			WebSocketError::Utf8Error(ref error) => Some(error),
			WebSocketError::WebSocketUrlError(ref error) => Some(error),
			_ => None,
		}
	}
}

impl From<io::Error> for WebSocketError {
	fn from(err: io::Error) -> WebSocketError {
		if err.kind() == io::ErrorKind::UnexpectedEof {
			return WebSocketError::NoDataAvailable;
		}
		WebSocketError::IoError(err)
	}
}

impl From<HttpError> for WebSocketError {
	fn from(err: HttpError) -> WebSocketError {
		WebSocketError::HttpError(err)
	}
}

impl From<ParseError> for WebSocketError {
	fn from(err: ParseError) -> WebSocketError {
		WebSocketError::UrlError(err)
	}
}

#[cfg(any(feature="sync-ssl", feature="async-ssl"))]
impl From<TlsError> for WebSocketError {
	fn from(err: TlsError) -> WebSocketError {
		WebSocketError::TlsError(err)
	}
}

#[cfg(any(feature="sync-ssl", feature="async-ssl"))]
impl<T> From<TlsHandshakeError<T>> for WebSocketError {
	fn from(err: TlsHandshakeError<T>) -> WebSocketError {
		match err {
			TlsHandshakeError::Failure(_) => WebSocketError::TlsHandshakeFailure,
			TlsHandshakeError::Interrupted(_) => WebSocketError::TlsHandshakeInterruption,
		}
	}
}

impl From<Utf8Error> for WebSocketError {
	fn from(err: Utf8Error) -> WebSocketError {
		WebSocketError::Utf8Error(err)
	}
}

#[cfg(feature="async")]
impl From<::codec::http::HttpCodecError> for WebSocketError {
	fn from(src: ::codec::http::HttpCodecError) -> Self {
		match src {
			::codec::http::HttpCodecError::Io(e) => WebSocketError::IoError(e),
			::codec::http::HttpCodecError::Http(e) => WebSocketError::HttpError(e),
		}
	}
}

impl From<WSUrlErrorKind> for WebSocketError {
	fn from(err: WSUrlErrorKind) -> WebSocketError {
		WebSocketError::WebSocketUrlError(err)
	}
}

impl From<HyperIntoWsError> for WebSocketError {
	fn from(err: HyperIntoWsError) -> WebSocketError {
		use self::HyperIntoWsError::*;
		use WebSocketError::*;
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
