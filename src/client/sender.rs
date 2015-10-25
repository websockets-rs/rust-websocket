//! The default implementation of a WebSocket Sender.

use std::io::Write;
use std::io::Result as IoResult;
use result::WebSocketResult;
use ws::dataframe::DataFrame;
use stream::WebSocketStream;
use stream::Shutdown;
use ws;

/// A Sender that wraps a Writer and provides a default implementation using
/// DataFrames and Messages.
pub struct Sender<W> {
	inner: W
}

impl<W> Sender<W> {
	/// Create a new WebSocketSender using the specified Writer.
	pub fn new(writer: W) -> Sender<W> {
		Sender {
			inner: writer
		}
	}
	/// Returns a reference to the underlying Writer.
	pub fn get_ref(&self) -> &W {
		&self.inner
	}
	/// Returns a mutable reference to the underlying Writer.
	pub fn get_mut(&mut self) -> &mut W {
		&mut self.inner
	}
}

impl Sender<WebSocketStream> {
    /// Closes the sender side of the connection, will cause all pending and future IO to
    /// return immediately with an appropriate value.
    pub fn shutdown(&mut self) -> IoResult<()> {
        self.inner.shutdown(Shutdown::Write)
    }

    /// Shuts down both Sender and Receiver, will cause all pending and future IO to
    /// return immediately with an appropriate value.
    pub fn shutdown_all(&mut self) -> IoResult<()> {
        self.inner.shutdown(Shutdown::Both)
    }
}

impl<W: Write> ws::Sender for Sender<W> {
	/// Sends a single data frame to the remote endpoint.
	fn send_dataframe<D>(&mut self, dataframe: &D) -> WebSocketResult<()>
	where D: DataFrame {
		dataframe.write_to(&mut self.inner, true)
	}
}
