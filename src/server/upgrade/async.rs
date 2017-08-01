//! Allows taking an existing stream of data and asynchronously convert it to a
//! websocket client.
//!
//! This module contains the trait that transforms stream into
//! an intermediate struct called `Upgrade` and the `Upgrade` struct itself.
//! The `Upgrade` struct is used to inspect details of the websocket connection
//! (e.g. what protocols it wants to use) and decide whether to accept or reject it.
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

/// An asynchronous websocket upgrade.
///
/// This struct is given when a connection is being upgraded to a websocket
/// request. It implements everything that a normal `WsUpgrade` struct does
/// along with the final functions to create websocket clients (although this
/// is done asynchronously).
///
/// # Example
///
/// ```rust,no_run
/// use websocket::async::{Core, TcpListener, TcpStream};
/// use websocket::async::futures::{Stream, Future};
/// use websocket::async::server::upgrade::IntoWs;
/// use websocket::sync::Client;
///
/// let mut core = Core::new().unwrap();
/// let handle = core.handle();
/// let addr = "127.0.0.1:80".parse().unwrap();
/// let listener = TcpListener::bind(&addr, &handle).unwrap();
///
/// let websocket_clients = listener
///     .incoming().map_err(|e| e.into())
///     .and_then(|(stream, _)| stream.into_ws().map_err(|e| e.3))
///     .map(|upgrade| {
///         if upgrade.protocols().iter().any(|p| p == "super-cool-proto") {
///             let accepted = upgrade
///                 .use_protocol("super-cool-proto")
///                 .accept()
///                 .map(|_| ()).map_err(|_| ());
///
///             handle.spawn(accepted);
///         } else {
///             let rejected = upgrade.reject()
///                 .map(|_| ()).map_err(|_| ());
///
///             handle.spawn(rejected);
///         }
///     });
/// ```
pub type Upgrade<S> = WsUpgrade<S, BytesMut>;

/// These are the extra functions given to `WsUpgrade` with the `async` feature
/// turned on. A type alias for this specialization of `WsUpgrade` lives in this
/// module under the name `Upgrade`.
impl<S> WsUpgrade<S, BytesMut>
    where S: Stream + 'static
{
	/// Asynchronously accept the websocket handshake, then create a client.
	/// This will asynchronously send a response accepting the connection
	/// and create a websocket client.
	pub fn accept(self) -> ClientNew<S> {
		self.internal_accept(None)
	}

	/// Asynchronously accept the websocket handshake, then create a client.
	/// This will asynchronously send a response accepting the connection
	/// with custom headers in the response and create a websocket client.
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

	/// Asynchronously send a rejection message and deconstruct `self`
	/// into it's original stream. The stream being returned is framed with the
	/// `HttpServerCodec` since that was used to send the rejection message.
	pub fn reject(self) -> Send<Framed<S, HttpServerCodec>> {
		self.internal_reject(None)
	}

	/// Asynchronously send a rejection message with custom headers and
	/// deconstruct `self` into it's original stream.
	///  The stream being returned is framed with the
	/// `HttpServerCodec` since that was used to send the rejection message.
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


/// Trait to take a stream or similar and attempt to recover the start of a
/// websocket handshake from it (asynchronously).
/// Should be used when a stream might contain a request for a websocket session.
///
/// If an upgrade request can be parsed, one can accept or deny the handshake with
/// the `WsUpgrade` struct.
/// Otherwise the original stream is returned along with an error.
///
/// Note: the stream is owned because the websocket client expects to own its stream.
///
/// This is already implemented for all async streams, which means all types with
/// `AsyncRead + AsyncWrite + 'static` (`'static` because the future wants to own
/// the stream).
///
/// # Example
///
/// ```rust,no_run
/// use websocket::async::{Core, TcpListener, TcpStream};
/// use websocket::async::futures::{Stream, Future};
/// use websocket::async::server::upgrade::IntoWs;
/// use websocket::sync::Client;
///
/// let mut core = Core::new().unwrap();
/// let handle = core.handle();
/// let addr = "127.0.0.1:80".parse().unwrap();
/// let listener = TcpListener::bind(&addr, &handle).unwrap();
///
/// let websocket_clients = listener
///     .incoming().map_err(|e| e.into())
///     .and_then(|(stream, _)| stream.into_ws().map_err(|e| e.3))
///     .map(|upgrade| {
///         if upgrade.protocols().iter().any(|p| p == "super-cool-proto") {
///             let accepted = upgrade
///                 .use_protocol("super-cool-proto")
///                 .accept()
///                 .map(|_| ()).map_err(|_| ());
///
///             handle.spawn(accepted);
///         } else {
///             let rejected = upgrade.reject()
///                 .map(|_| ()).map_err(|_| ());
///
///             handle.spawn(rejected);
///         }
///     });
/// ```
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
