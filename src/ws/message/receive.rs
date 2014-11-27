use super::dataframe::{WebSocketOpcode, ReadWebSocketDataFrame};
use super::WebSocketMessage;
use super::mask::mask_data;
use std::iter::Iterator;
use std::io::net::tcp::TcpStream;
use std::io::{IoResult, IoError, IoErrorKind};
use std::option::Option;
use std::str::from_utf8;

/// Represents a WebSocket receiver which can receive data from the remote endpoint.
pub struct WebSocketReceiver {
	stream: TcpStream,
	data: Vec<u8>,
}

impl WebSocketReceiver {
	/// Wait for and accept a message (subjected to the underlying stream timeout).
	/// If the received message is fragmented, this function will not return
	/// until the final fragment has been received.
	pub fn receive_message(&mut self) -> IoResult<WebSocketMessage> {
		let dataframe = try!(self.stream.read_websocket_dataframe());
		let mut data = dataframe.data.clone();
		match dataframe.mask {
			Some(key) => { data = mask_data(key, dataframe.data.as_slice()); }
			None => { }
		}
		
		self.data.push_all(data.as_slice());
		
		if !dataframe.finished {
			loop {
				let df = try!(self.stream.read_websocket_dataframe());
				match df.opcode {
					WebSocketOpcode::Continuation => {
						let mut data = df.data.clone();
						match df.mask {
							Some(key) => { data = mask_data(key, data.as_slice()); }
							None => { }
						}
						self.data.push_all(data.as_slice());
					}
					WebSocketOpcode::Close => {
						return Ok(WebSocketMessage::Close);
					}
					WebSocketOpcode::Ping => {
						return Ok(WebSocketMessage::Ping);
					}
					WebSocketOpcode::Pong => {
						return Ok(WebSocketMessage::Pong);
					}
					_ => {
						return Err(IoError {
							kind: IoErrorKind::InvalidInput,
							desc: "Unexpected non-continuation dataframe",
							detail: None,
						});
					}
				}
				if df.finished { break; }
			}
		}
		
		data = self.data.clone();
		self.data = Vec::new();
		
		match dataframe.opcode {
			WebSocketOpcode::Continuation => {
				Err(IoError {
					kind: IoErrorKind::InvalidInput,
					desc: "Unexpected continuation dataframe",
					detail: None,
				})
			}
			WebSocketOpcode::Text => {
				let s = try!(from_utf8(data.as_slice()).ok_or(
					IoError {
						kind: IoErrorKind::InvalidInput,
						desc: "No host specified",
						detail: None,
					}
				));
				Ok(WebSocketMessage::Text(s.to_string()))
			}
			WebSocketOpcode::Binary => { Ok(WebSocketMessage::Binary(data)) }
			WebSocketOpcode::Close => { Ok(WebSocketMessage::Close) }
			WebSocketOpcode::Ping => { Ok(WebSocketMessage::Ping) }
			WebSocketOpcode::Pong => { Ok(WebSocketMessage::Pong) }
			_ => {
				Err(IoError {
					kind: IoErrorKind::InvalidInput,
					desc: "Unsupported opcode received",
					detail: None,
				})
			}
		}
	}

	/// Returns an iterator over the incoming messages for/from this client
	pub fn incoming(self) -> IncomingMessages {
		IncomingMessages {
			inc: self,
		}
	}
}

pub fn new_receiver(stream: TcpStream) -> WebSocketReceiver {
	WebSocketReceiver {
		stream: stream,
		data: Vec::new(),
	}
}

/// An iterator over incoming messages
pub struct IncomingMessages {
	inc: WebSocketReceiver,
}

impl Iterator<IoResult<WebSocketMessage>> for IncomingMessages {
	fn next(&mut self) -> Option<IoResult<WebSocketMessage>> {
		Some(self.inc.receive_message())
	}
}