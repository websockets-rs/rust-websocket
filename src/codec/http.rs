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
pub struct HttpClientCodec;

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
		let (response, bytes_read) = {
			let mut reader = BufReader::new(&*src as &[u8]);
			let res = match parse_response(&mut reader) {
				Err(hyper::Error::Io(ref e)) if e.kind() == io::ErrorKind::UnexpectedEof => {
					return Ok(None)
				}
				Err(hyper::Error::TooLarge) => return Ok(None),
				Err(e) => return Err(e.into()),
				Ok(r) => r,
			};
			let (_, _, pos, _) = reader.into_parts();
			(res, pos)
		};

		src.split_to(bytes_read);
		Ok(Some(response))
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
		let (response, bytes_read) = {
			let mut reader = BufReader::new(&*src as &[u8]);
			let res = match parse_request(&mut reader) {
				Err(hyper::Error::Io(ref e)) if e.kind() == io::ErrorKind::UnexpectedEof => {
					return Ok(None)
				}
				Err(hyper::Error::TooLarge) => return Ok(None),
				Err(e) => return Err(e.into()),
				Ok(r) => r,
			};
			let (_, _, pos, _) = reader.into_parts();
			(res, pos)
		};

		src.split_to(bytes_read);
		Ok(Some(response))
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
