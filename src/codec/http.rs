//! Send HTTP requests and responses asynchronously.
//!
//! This module has both an `HttpClientCodec` for an async HTTP client and an
//! `HttpServerCodec` for an async HTTP server.
use std::io::{self, Write};
use std::error::Error;
use std::fmt::{self, Formatter, Display};
use hyper;
use hyper::http::h1::Incoming;
use hyper::http::h1::parse_response;
use hyper::http::h1::parse_request;
use hyper::http::RawStatus;
use hyper::status::StatusCode;
use hyper::method::Method;
use hyper::uri::RequestUri;
use hyper::buffer::BufReader;
use tokio_io::codec::{Decoder, Encoder};
use bytes::BytesMut;
use bytes::BufMut;

#[derive(Copy, Clone, Debug)]
///```rust,no_run
///# extern crate tokio_core;
///# extern crate tokio_io;
///# extern crate websocket;
///# extern crate hyper;
///# use websocket::codec::http::HttpClientCodec;
///# use websocket::async::futures::Future;
///# use websocket::async::futures::Sink;
///# use tokio_core::net::TcpStream;
///# use tokio_core::reactor::Core;
///# use tokio_io::AsyncRead;
///# use hyper::http::h1::Incoming;
///# use hyper::version::HttpVersion;
///# use hyper::header::Headers;
///# use hyper::method::Method;
///# use hyper::uri::RequestUri;
///
///# fn main() {
///let core = Core::new().unwrap();
///
///let f = TcpStream::connect(&"crouton.net".parse().unwrap(), &core.handle())
///    .and_then(|s| {
///        Ok(s.framed(HttpClientCodec))
///    })
///    .and_then(|s| {
///        s.send(Incoming {
///            version: HttpVersion::Http11,
///            subject: (Method::Get, RequestUri::AbsolutePath("/".to_string())),
///            headers: Headers::new(),
///        })
///    });
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
		let request = format!("{} {} {}\r\n{}\r\n",
		                      item.subject.0,
		                      item.subject.1,
		                      item.version,
		                      item.headers);
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
						return Ok(None)
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

#[derive(Debug)]
pub enum HttpCodecError {
	Io(io::Error),
	Http(hyper::Error),
}

impl Display for HttpCodecError {
	fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
		fmt.write_str(self.description())
	}
}

impl Error for HttpCodecError {
	fn description(&self) -> &str {
		match *self {
			HttpCodecError::Io(ref e) => e.description(),
			HttpCodecError::Http(ref e) => e.description(),
		}
	}

	fn cause(&self) -> Option<&Error> {
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
