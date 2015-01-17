//! Module containing a useful stream type for WebSocket connections

use std::io::TcpStream;
use openssl::ssl::SslStream;

use dataframe::WebSocketDataFrame;
use result::WebSocketResult;
use ws::Sender;
use ws::Receiver;
use ws::util::{Local, Remote};

/// Represents a stream capable of carrying a WebSocket connection.
pub enum WebSocketStream<L> {
	/// A normal, non-secure connection over TCP.
	TcpStream(TcpStream),
	/// A secure, TLS-backed connection.
	SslStream(SslStream<TcpStream>),
	#[test]
	/// A dummy connection for debugging.
	Dummy,
}

impl Sender<WebSocketDataFrame> for WebSocketStream<Local> {
	fn send_dataframe(&mut self, dataframe: WebSocketDataFrame) -> WebSocketResult<()> {
		Ok(())
	}
}

impl Sender<WebSocketDataFrame> for WebSocketStream<Remote> {
	fn send_dataframe(&mut self, dataframe: WebSocketDataFrame) -> WebSocketResult<()> {
		Ok(())
	}
}