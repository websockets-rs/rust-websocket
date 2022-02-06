//! Send HTTP requests and responses asynchronously.
//!
//! This module has both an `HttpClientCodec` for an async HTTP client and an
//! `HttpServerCodec` for an async HTTP server.
use bytes::BufMut;
use bytes::BytesMut;
use hyper;
use hyper::buffer::BufReader;
use hyper::http::h1::parse_request;
use hyper::http::h1::parse_response;
use hyper::http::h1::Incoming;
use hyper::http::RawStatus;
use hyper::method::Method;
use hyper::status::StatusCode;
use hyper::uri::RequestUri;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io::{self, Write};
use tokio_codec::{Decoder, Encoder};

#[derive(Copy, Clone, Debug)]
///A codec to be used with `tokio` codecs that can serialize HTTP requests and
///deserialize HTTP responses. One can use this on it's own without websockets to
///make a very bare async HTTP server.
///
///# Example
///```rust,no_run
///# extern crate tokio;
///# extern crate websocket;
///# extern crate hyper;
///use websocket::r#async::HttpClientCodec;
///# use websocket::r#async::futures::{Future, Sink, Stream};
///# use tokio::net::TcpStream;
///# use tokio::codec::Decoder;
///# use hyper::http::h1::Incoming;
///# use hyper::version::HttpVersion;
///# use hyper::header::Headers;
///# use hyper::method::Method;
///# use hyper::uri::RequestUri;
///
///# fn main() {
///let mut runtime = tokio::runtime::Builder::new().build().unwrap();
///let addr = "crouton.net".parse().unwrap();
///
///let f = TcpStream::connect(&addr)
///    .and_then(|s| {
///        Ok(HttpClientCodec.framed(s))
///    })
///    .and_then(|s| {
///        s.send(Incoming {
///            version: HttpVersion::Http11,
///            subject: (Method::Get, RequestUri::AbsolutePath("/".to_string())),
///            headers: Headers::new(),
///        })
///    })
///    .map_err(|e| e.into())
///    .and_then(|s| s.into_future().map_err(|(e, _)| e))
///    .map(|(m, _)| println!("You got a crouton: {:?}", m));
///
///runtime.block_on(f).unwrap();
///# }
///```
pub struct HttpClientCodec;

fn split_off_http(src: &mut BytesMut) -> Option<BytesMut> {
	match src.windows(4).position(|i| i == b"\r\n\r\n") {
		Some(p) => Some(src.split_to(p + 4)),
		None => None,
	}
}

impl Encoder for HttpClientCodec {
	type Item = Incoming<(Method, RequestUri)>;
	type Error = io::Error;

	fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
		// TODO: optomize this!
		let request = format!(
			"{} {} {}\r\n{}\r\n",
			item.subject.0, item.subject.1, item.version, item.headers
		);
		let byte_len = request.as_bytes().len();
		if byte_len > dst.remaining_mut() {
			dst.reserve(byte_len);
		}
		dst.writer().write(request.as_bytes()).map(|_| ())
	}
}

impl Decoder for HttpClientCodec {
	type Item = Incoming<RawStatus>;
	type Error = HttpCodecError;

	fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
		// check if we get a request from hyper
		// TODO: this is ineffecient, but hyper does not give us a better way to parse
		match split_off_http(src) {
			Some(buf) => {
				let mut reader = BufReader::with_capacity(&*buf as &[u8], buf.len());
				let res = match parse_response(&mut reader) {
					Err(hyper::Error::Io(ref e)) if e.kind() == io::ErrorKind::UnexpectedEof => {
						return Ok(None);
					}
					Err(hyper::Error::TooLarge) => return Ok(None),
					Err(e) => return Err(e.into()),
					Ok(r) => r,
				};
				Ok(Some(res))
			}
			None => Ok(None),
		}
	}
}

///A codec that can be used with streams implementing `AsyncRead + AsyncWrite`
///that can serialize HTTP responses and deserialize HTTP requests. Using this
///with an async `TcpStream` will give you a very bare async HTTP server.
///
///This crate sends out one HTTP request / response in order to perform the websocket
///handshake then never talks HTTP again. Because of this an async HTTP implementation
///is needed.
///
///# Example
///
///```rust,no_run
///# extern crate tokio;
///# extern crate websocket;
///# extern crate hyper;
///# use std::io;
///use websocket::r#async::HttpServerCodec;
///# use websocket::r#async::futures::{Future, Sink, Stream};
///# use tokio::net::TcpStream;
///# use tokio::codec::Decoder;
///# use hyper::http::h1::Incoming;
///# use hyper::version::HttpVersion;
///# use hyper::header::Headers;
///# use hyper::method::Method;
///# use hyper::uri::RequestUri;
///# use hyper::status::StatusCode;
///# fn main() {
///
///let mut runtime = tokio::runtime::Builder::new().build().unwrap();
///let addr = "nothing-to-see-here.com".parse().unwrap();
///
///let f = TcpStream::connect(&addr)
///   .map(|s| HttpServerCodec.framed(s))
///   .map_err(|e| e.into())
///   .and_then(|s| s.into_future().map_err(|(e, _)| e))
///   .and_then(|(m, s)| match m {
///       Some(ref m) if m.subject.0 == Method::Get => Ok(s),
///       _ => panic!(),
///   })
///   .and_then(|stream| {
///       stream
///          .send(Incoming {
///               version: HttpVersion::Http11,
///               subject: StatusCode::NotFound,
///               headers: Headers::new(),
///           })
///           .map_err(|e| e.into())
///   });
///
///runtime.block_on(f).unwrap();
///# }
///```
#[derive(Copy, Clone, Debug)]
pub struct HttpServerCodec;

impl Encoder for HttpServerCodec {
	type Item = Incoming<StatusCode>;
	type Error = io::Error;

	fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
		// TODO: optomize this!
		let response = format!("{} {}\r\n{}\r\n", item.version, item.subject, item.headers);
		let byte_len = response.as_bytes().len();
		if byte_len > dst.remaining_mut() {
			dst.reserve(byte_len);
		}
		dst.writer().write(response.as_bytes()).map(|_| ())
	}
}

impl Decoder for HttpServerCodec {
	type Item = Incoming<(Method, RequestUri)>;
	type Error = HttpCodecError;

	fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
		// check if we get a request from hyper
		// TODO: this is ineffecient, but hyper does not give us a better way to parse
		match split_off_http(src) {
			Some(buf) => {
				let mut reader = BufReader::with_capacity(&*buf as &[u8], buf.len());
				let res = match parse_request(&mut reader) {
					Err(hyper::Error::Io(ref e)) if e.kind() == io::ErrorKind::UnexpectedEof => {
						return Ok(None);
					}
					Err(hyper::Error::TooLarge) => return Ok(None),
					Err(e) => return Err(e.into()),
					Ok(r) => r,
				};
				Ok(Some(res))
			}
			None => Ok(None),
		}
	}
}

/// Any error that can happen during the writing or parsing of HTTP requests
/// and responses. This consists of HTTP parsing errors (the `Http` variant) and
/// errors that can occur when writing to IO (the `Io` variant).
#[derive(Debug)]
pub enum HttpCodecError {
	/// An error that occurs during the writing or reading of HTTP data
	/// from a socket.
	Io(io::Error),
	/// An error that occurs during the parsing of an HTTP request or response.
	Http(hyper::Error),
}

impl Display for HttpCodecError {
	fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
		match self {
			HttpCodecError::Io(e) => fmt.write_str(e.to_string().as_str()),
			HttpCodecError::Http(e) => fmt.write_str(e.to_string().as_str()),
		}
	}
}

impl Error for HttpCodecError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match *self {
			HttpCodecError::Io(ref error) => Some(error),
			HttpCodecError::Http(ref error) => Some(error),
		}
	}
}

impl From<io::Error> for HttpCodecError {
	fn from(err: io::Error) -> HttpCodecError {
		HttpCodecError::Io(err)
	}
}

impl From<hyper::Error> for HttpCodecError {
	fn from(err: hyper::Error) -> HttpCodecError {
		HttpCodecError::Http(err)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::stream::ReadWritePair;
	use futures::{Future, Sink, Stream};
	use hyper::header::Headers;
	use hyper::version::HttpVersion;
	use std::io::Cursor;
	use tokio::runtime::current_thread::Builder;

	#[test]
	fn test_client_http_codec() {
		let mut runtime = Builder::new().build().unwrap();
		let response = "HTTP/1.1 404 Not Found\r\n\r\npssst extra data here";
		let input = Cursor::new(response.as_bytes());
		let output = Cursor::new(Vec::new());

		let f = HttpClientCodec
			.framed(ReadWritePair(input, output))
			.send(Incoming {
				version: HttpVersion::Http11,
				subject: (Method::Get, RequestUri::AbsolutePath("/".to_string())),
				headers: Headers::new(),
			})
			.map_err(|e| e.into())
			.and_then(|s| s.into_future().map_err(|(e, _)| e))
			.and_then(|(m, _)| match m {
				Some(ref m) if StatusCode::from_u16(m.subject.0) == StatusCode::NotFound => Ok(()),
				_ => Err(io::Error::new(io::ErrorKind::Other, "test failed").into()),
			});
		runtime.block_on(f).unwrap();
	}

	#[test]
	fn test_server_http_codec() {
		let mut runtime = Builder::new().build().unwrap();
		let request = "\
		               GET / HTTP/1.0\r\n\
		               Host: www.rust-lang.org\r\n\
		               \r\n\
		               "
		.as_bytes();
		let input = Cursor::new(request);
		let output = Cursor::new(Vec::new());

		let f = HttpServerCodec
			.framed(ReadWritePair(input, output))
			.into_future()
			.map_err(|(e, _)| e)
			.and_then(|(m, s)| match m {
				Some(ref m) if m.subject.0 == Method::Get => Ok(s),
				_ => Err(io::Error::new(io::ErrorKind::Other, "test failed").into()),
			})
			.and_then(|s| {
				s.send(Incoming {
					version: HttpVersion::Http11,
					subject: StatusCode::NotFound,
					headers: Headers::new(),
				})
				.map_err(|e| e.into())
			});
		runtime.block_on(f).unwrap();
	}
}
