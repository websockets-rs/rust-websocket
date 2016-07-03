//! Allows you to take an existing request or stream of data and convert it into a
//! WebSocket client.
extern crate hyper;
extern crate openssl;

use super::super::stream::Stream;

/// Any error that could occur when attempting
/// to parse data into a websocket upgrade request
pub enum IntoWsError {
	/// If the request was not actually asking for a websocket connection
	RequestIsNotUpgrade,
}

/// Intermediate representation of a half created websocket session.
/// Should be used to examine the client's handshake
/// accept the protocols requested, route the path, etc.
///
/// Users should then call `accept` or `deny` to complete the handshake
/// and start a session.
pub struct WsUpgrade<S>
where S: Stream,
{
	stream: S,
}

impl<S> WsUpgrade<S>
where S: Stream,
{
	fn from_stream(inner: S) -> Self {
		WsUpgrade {
			stream: inner,
		}
	}

	fn unwrap(self) -> S {
		self.stream
	}

	fn accept(self) {
		unimplemented!();
	}

	fn reject(self) -> S {
		unimplemented!();
	}
}

/// Trait to take a stream or similar and attempt to recover the start of a
/// websocket handshake from it.
/// Should be used when a stream might contain a request for a websocket session.
///
/// If an upgrade request can be parsed, one can accept or deny the handshake with
/// the `WsUpgrade` struct.
/// Otherwise the original stream is returned along with an error.
///
/// Note: the stream is owned because the websocket client expects to own its stream.
pub trait IntoWs<O>
{
    /// Attempt to parse the start of a Websocket handshake, later with the  returned
    /// `WsUpgrade` struct, call `accept to start a websocket client, and `reject` to
    /// send a handshake rejection response.
	fn into_ws(self) -> Result<WsUpgrade<S>, (O, IntoWsError)>;
}

