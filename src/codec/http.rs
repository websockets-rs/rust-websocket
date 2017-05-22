use hyper;
use hyper::http::h1::Incoming;
use hyper::http::h1::parse_response;
use hyper::http::RawStatus;
use hyper::method::Method;
use hyper::uri::RequestUri;
use hyper::buffer::BufReader;
use std::io::{self, Write};
use tokio_io::codec::{Decoder, Encoder};
use bytes::BytesMut;
use bytes::BufMut;
use result::WebSocketError;

#[derive(Copy, Clone, Debug)]
pub struct HttpCodec;

impl Encoder for HttpCodec {
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

impl Decoder for HttpCodec {
	type Item = Incoming<RawStatus>;
	type Error = WebSocketError;

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
