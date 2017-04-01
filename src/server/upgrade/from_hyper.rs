//! Upgrade a hyper connection to a websocket one.
//!
//! Using this method, one can start a hyper server and check if each request
//! is a websocket upgrade request, if so you can use websockets and hyper on the
//! same port!
//!
//! ```rust,no_run
//! # extern crate hyper;
//! # extern crate websocket;
//! # fn main() {
//! use hyper::server::{Server, Request, Response};
//! use websocket::Message;
//! use websocket::server::upgrade::IntoWs;
//! use websocket::server::upgrade::from_hyper::HyperRequest;
//!
//! Server::http("0.0.0.0:80").unwrap().handle(move |req: Request, res: Response| {
//!     match HyperRequest(req).into_ws() {
//!         Ok(upgrade) => {
//!             // `accept` sends a successful handshake, no need to worry about res
//!             let mut client = match upgrade.accept() {
//!                 Ok(c) => c,
//!                 Err(_) => panic!(),
//!             };
//!
//!             client.send_message(&Message::text("its free real estate"));
//!         },
//!
//!         Err((request, err)) => {
//!             // continue using the request as normal, "echo uri"
//!             res.send(b"Try connecting over ws instead.").unwrap();
//!         },
//!     };
//! })
//! .unwrap();
//! # }
//! ```

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

/// A hyper request is implicitly defined as a stream from other `impl`s of Stream.
/// Until trait impl specialization comes along, we use this struct to differentiate
/// a hyper request (which already has parsed headers) from a normal stream.
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
