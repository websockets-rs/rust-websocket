use super::{Buffer, HyperIntoWsError, WsUpgrade, Request, validate};
use std::io::{self, ErrorKind};
use hyper::header::Headers;
use stream::AsyncStream;
use futures::{Stream, Future};
use codec::http::HttpServerCodec;
use bytes::BytesMut;

pub trait AsyncIntoWs {
	/// The type of stream this upgrade process is working with (TcpStream, etc.)
	type Stream: AsyncStream;
	/// An error value in case the stream is not asking for a websocket connection
	/// or something went wrong. It is common to also include the stream here.
	type Error;
	/// Attempt to read and parse the start of a Websocket handshake, later
	/// with the  returned `WsUpgrade` struct, call `accept to start a
	/// websocket client, and `reject` to send a handshake rejection response.
	///
	/// Note: this is the asynchronous version, meaning it will not block when
	/// trying to read a request.
	fn into_ws(self) -> Box<Future<Item = WsUpgrade<Self::Stream>, Error = Self::Error>>;
}

impl<S> AsyncIntoWs for S
    where S: AsyncStream + 'static
{
	type Stream = S;
	type Error = (S, Option<Request>, Option<BytesMut>, HyperIntoWsError);

	fn into_ws(self) -> Box<Future<Item = WsUpgrade<Self::Stream>, Error = Self::Error>> {
		let future = self.framed(HttpServerCodec)
          .into_future()
          .map_err(|(e, s)| {
              let (stream, buffer) = s.into_parts();
              (stream, None, Some(buffer), e.into())
          })
          .and_then(|(m, s)| {
              let (stream, buffer) = s.into_parts();
              if let Some(msg) = m {
                  match validate(&msg.subject.0, &msg.version, &msg.headers) {
                      Ok(()) => Ok((msg, stream, buffer)),
                      Err(e) => Err((stream, None, Some(buffer), e)),
                  }
              } else {
                  let err = HyperIntoWsError::Io(io::Error::new(
                      ErrorKind::ConnectionReset,
                  "Connection dropped before handshake could be read"));
                  Err((stream, None, Some(buffer), err))
              }
          })
          .map(|(m, stream, buffer)| {
              WsUpgrade {
                  headers: Headers::new(),
                  stream: stream,
                  request: m,
                  buffer: Some(Buffer {
                      buf: unimplemented!(),
                      pos: 0,
                      cap: buffer.capacity(),
                  }),
              }
          });
		Box::new(future)
	}
}
