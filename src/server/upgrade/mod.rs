//! Allows you to take an existing request or stream of data and convert it into a
//! WebSocket client.
use std::net::TcpStream;
use stream::{
	Stream,
	AsTcpStream,
};

pub mod hyper;

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
	request: hyper::Request,
}

impl<S> WsUpgrade<S>
where S: Stream,
{
	pub fn accept(self) {
		unimplemented!();
	}

	pub fn reject(self) -> S {
		unimplemented!();
	}

	pub fn into_stream(self) -> S {
		unimplemented!();
	}
}

impl<S> WsUpgrade<S>
where S: Stream + AsTcpStream,
{
	pub fn tcp_stream(&self) -> &TcpStream {
		self.stream.as_tcp()
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
pub trait IntoWs {
	type Stream: Stream;
	type Error;
	/// Attempt to parse the start of a Websocket handshake, later with the  returned
	/// `WsUpgrade` struct, call `accept to start a websocket client, and `reject` to
	/// send a handshake rejection response.
	fn into_ws(mut self) -> Result<WsUpgrade<Self::Stream>, Self::Error>;
}

