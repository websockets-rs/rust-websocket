//! The default implementation of a WebSocket Sender.

use std::io::Write;
use std::io::Result as IoResult;
use result::WebSocketResult;
use ws::dataframe::DataFrame;
use stream::sync::AsTcpStream;
use ws;
use ws::sender::Sender as SenderTrait;
pub use stream::sync::Shutdown;

/// A writer that bundles a stream with a serializer to send the messages.
/// This is used in the client's `.split()` function as the writing component.
///
/// It can also be useful to use a websocket connection without a handshake.
pub struct Writer<W> {
	/// The stream that websocket messages will be written to
	pub stream: W,
	/// The serializer that will be used to serialize the messages
	pub sender: Sender,
}

impl<W> Writer<W>
    where W: Write
{
	/// Sends a single data frame to the remote endpoint.
	pub fn send_dataframe<D>(&mut self, dataframe: &D) -> WebSocketResult<()>
		where D: DataFrame,
		      W: Write
	{
		self.sender.send_dataframe(&mut self.stream, dataframe)
	}

	/// Sends a single message to the remote endpoint.
	pub fn send_message<M>(&mut self, message: &M) -> WebSocketResult<()>
		where M: ws::Message
	{
		self.sender.send_message(&mut self.stream, message)
	}
}

impl<S> Writer<S>
    where S: AsTcpStream + Write
{
	/// Closes the sender side of the connection, will cause all pending and future IO to
	/// return immediately with an appropriate value.
	pub fn shutdown(&self) -> IoResult<()> {
		self.stream.as_tcp().shutdown(Shutdown::Write)
	}

	/// Shuts down both Sender and Receiver, will cause all pending and future IO to
	/// return immediately with an appropriate value.
	pub fn shutdown_all(&self) -> IoResult<()> {
		self.stream.as_tcp().shutdown(Shutdown::Both)
	}
}

/// A Sender that wraps a Writer and provides a default implementation using
/// DataFrames and Messages.
pub struct Sender {
	mask: bool,
}

impl Sender {
	/// Create a new WebSocketSender using the specified Writer.
	pub fn new(mask: bool) -> Sender {
		Sender { mask: mask }
	}
}

impl ws::Sender for Sender {
	fn is_masked(&self) -> bool {
		self.mask
	}
}
