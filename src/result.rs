#![stable]
//! The result type used within Rust-WebSocket

use std::io;
use std::str::Utf8Error;
use std::error::{Error, FromError};
use std::fmt;
use openssl::ssl::error::SslError;
use hyper::HttpError;
use url::ParseError;
use byteorder;

/// The type used for WebSocket results
pub type WebSocketResult<T> = Result<T, WebSocketError>;

/// Represents a WebSocket error
#[derive(Debug, PartialEq, Clone)]
pub enum WebSocketError {
	/// A WebSocket protocol error
	ProtocolError(String),
	/// Invalid WebSocket request error
	RequestError(String),
	/// Invalid WebSocket response error
	ResponseError(String),
	/// Invalid WebSocket data frame error
	DataFrameError(String),
	/// No data available
	NoDataAvailable,
	/// An input/output error
	IoError(io::Error),
	/// An HTTP parsing error
	HttpError(HttpError),
	/// A URL parsing error
	UrlError(ParseError),
	/// An SSL error
	SslError(SslError),
	/// A UTF-8 error
	Utf8Error(Utf8Error),
}

impl fmt::Display for WebSocketError {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		try!(fmt.write_str("WebSocketError: "));
		try!(fmt.write_str(self.description()));
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
			WebSocketError::SslError(_) => "SSL failure",
			WebSocketError::Utf8Error(_) => "UTF-8 failure",
		}
	}

	fn cause(&self) -> Option<&Error> {
		match *self {
			WebSocketError::IoError(ref error) => Some(error),
			WebSocketError::HttpError(ref error) => Some(error),
			WebSocketError::UrlError(ref error) => Some(error),
			WebSocketError::SslError(ref error) => Some(error),
			WebSocketError::Utf8Error(ref error) => Some(error),
			_ => None,
		}
	}
}

impl FromError<io::Error> for WebSocketError {
	fn from_error(err: io::Error) -> WebSocketError {
		WebSocketError::IoError(err)
	}
}

impl FromError<HttpError> for WebSocketError {
	fn from_error(err: HttpError) -> WebSocketError {
		WebSocketError::HttpError(err)
	}
}

impl FromError<ParseError> for WebSocketError {
	fn from_error(err: ParseError) -> WebSocketError {
		WebSocketError::UrlError(err)
	}
}

impl FromError<SslError> for WebSocketError {
	fn from_error(err: SslError) -> WebSocketError {
		WebSocketError::SslError(err)
	}
}

impl FromError<Utf8Error> for WebSocketError {
	fn from_error(err: Utf8Error) -> WebSocketError {
		WebSocketError::Utf8Error(err)
	}
}

impl FromError<byteorder::Error> for WebSocketError {
	fn from_error(err: byteorder::Error) -> WebSocketError {
		match err {
			byteorder::Error::UnexpectedEOF => WebSocketError::NoDataAvailable,
			byteorder::Error::Io(err) => FromError::from_error(err)
		}
	}
}
