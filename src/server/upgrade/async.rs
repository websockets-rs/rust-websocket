use super::{HyperIntoWsError, WsUpgrade, Request, validate};
use std::io::{self, ErrorKind};
use tokio_io::codec::{Framed, FramedParts};
use hyper::header::Headers;
use hyper::http::h1::Incoming;
use hyper::status::StatusCode;
use stream::async::Stream;
use futures::{Sink, Future};
use futures::Stream as StreamTrait;
use futures::sink::Send;
use codec::http::HttpServerCodec;
use codec::ws::{MessageCodec, Context};
use bytes::BytesMut;
use client::async::ClientNew;

pub type Upgrade<S> = WsUpgrade<S, BytesMut>;

/// TODO: docs, THIS IS THTE ASYNC ONE
impl<S> WsUpgrade<S, BytesMut>
    where S: Stream + 'static
{
	pub fn accept(self) -> ClientNew<S> {
		self.internal_accept(None)
	}

	pub fn accept_with(self, custom_headers: &Headers) -> ClientNew<S> {
		self.internal_accept(Some(custom_headers))
	}

	fn internal_accept(mut self, custom_headers: Option<&Headers>) -> ClientNew<S> {
		let status = self.prepare_headers(custom_headers);
		let WsUpgrade { headers, stream, request, buffer } = self;

		let duplex = Framed::from_parts(FramedParts {
		                                    inner: stream,
		                                    readbuf: buffer,
		                                    writebuf: BytesMut::with_capacity(0),
		                                },
		                                HttpServerCodec);

		let future = duplex.send(Incoming {
		                             version: request.version,
		                             subject: status,
		                             headers: headers.clone(),
		                         })
		                   .map(move |s| {
			                        let codec = MessageCodec::default(Context::Server);
			                        let client = Framed::from_parts(s.into_parts(), codec);
			                        (client, headers)
			                       })
		                   .map_err(|e| e.into());
		Box::new(future)
	}

	pub fn reject(self) -> Send<Framed<S, HttpServerCodec>> {
		self.internal_reject(None)
	}

	pub fn reject_with(self, headers: &Headers) -> Send<Framed<S, HttpServerCodec>> {
		self.internal_reject(Some(headers))
	}

	fn internal_reject(mut self, headers: Option<&Headers>) -> Send<Framed<S, HttpServerCodec>> {
		if let Some(custom) = headers {
			self.headers.extend(custom.iter());
		}
		let duplex = Framed::from_parts(FramedParts {
		                                    inner: self.stream,
		                                    readbuf: self.buffer,
		                                    writebuf: BytesMut::with_capacity(0),
		                                },
		                                HttpServerCodec);
		duplex.send(Incoming {
		                version: self.request.version,
		                subject: StatusCode::BadRequest,
		                headers: self.headers,
		            })
	}
}


pub trait IntoWs {
	/// The type of stream this upgrade process is working with (TcpStream, etc.)
	type Stream: Stream;
	/// An error value in case the stream is not asking for a websocket connection
	/// or something went wrong. It is common to also include the stream here.
	type Error;
	/// Attempt to read and parse the start of a Websocket handshake, later
	/// with the  returned `WsUpgrade` struct, call `accept to start a
	/// websocket client, and `reject` to send a handshake rejection response.
	///
	/// Note: this is the asynchronous version, meaning it will not block when
	/// trying to read a request.
	fn into_ws(self) -> Box<Future<Item = Upgrade<Self::Stream>, Error = Self::Error>>;
}

impl<S> IntoWs for S
    where S: Stream + 'static
{
	type Stream = S;
	type Error = (S, Option<Request>, BytesMut, HyperIntoWsError);

	fn into_ws(self) -> Box<Future<Item = Upgrade<Self::Stream>, Error = Self::Error>> {
		let future = self.framed(HttpServerCodec)
          .into_future()
          .map_err(|(e, s)| {
              let FramedParts { inner, readbuf, .. } = s.into_parts();
              (inner, None, readbuf, e.into())
          })
          .and_then(|(m, s)| {
              let FramedParts { inner, readbuf, .. } = s.into_parts();
              if let Some(msg) = m {
                  match validate(&msg.subject.0, &msg.version, &msg.headers) {
                      Ok(()) => Ok((msg, inner, readbuf)),
                      Err(e) => Err((inner, None, readbuf, e)),
                  }
              } else {
                  let err = HyperIntoWsError::Io(io::Error::new(
                      ErrorKind::ConnectionReset,
                  "Connection dropped before handshake could be read"));
                  Err((inner, None, readbuf, err))
              }
          })
          .map(|(m, stream, buffer)| {
              WsUpgrade {
                  headers: Headers::new(),
                  stream: stream,
                  request: m,
                  buffer: buffer,
              }
          });
		Box::new(future)
	}
}
