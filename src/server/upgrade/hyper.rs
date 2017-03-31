extern crate hyper;

use hyper::net::NetworkStream;
use super::{IntoWs, WsUpgrade, Buffer};

pub use hyper::http::h1::Incoming;
pub use hyper::method::Method;
pub use hyper::version::HttpVersion;
pub use hyper::uri::RequestUri;
pub use hyper::buffer::BufReader;
use hyper::server::Request;
pub use hyper::header::{Headers, Upgrade, ProtocolName, Connection, ConnectionOption};

use super::validate;
use super::HyperIntoWsError;

pub struct HyperRequest<'a, 'b: 'a>(pub Request<'a, 'b>);

impl<'a, 'b> IntoWs for HyperRequest<'a, 'b> {
	type Stream = &'a mut &'b mut NetworkStream;
	type Error = (Request<'a, 'b>, HyperIntoWsError);

	fn into_ws(self) -> Result<WsUpgrade<Self::Stream>, Self::Error> {
		if let Err(e) = validate(&self.0.method, &self.0.version, &self.0.headers) {
			return Err((self.0, e));
		}

		let (_, method, headers, uri, version, reader) =
			self.0.deconstruct();

		let reader = reader.into_inner();
		let (buf, pos, cap) = reader.take_buf();
		let stream = reader.get_mut();

		Ok(WsUpgrade {
		       headers: Headers::new(),
		       stream: stream,
		       buffer: Some(Buffer {
		                        buf: buf,
		                        pos: pos,
		                        cap: cap,
		                    }),
		       request: Incoming {
		           version: version,
		           headers: headers,
		           subject: (method, uri),
		       },
		   })
	}
}
