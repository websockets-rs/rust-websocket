use super::dataframe::{WebSocketOpcode, ReadWebSocketDataFrame};
use super::WebSocketMessage;
use super::mask::mask_data;
use std::iter::Iterator;
use std::io::net::tcp::TcpStream;
use std::io::{IoResult, IoError, IoErrorKind};
use std::option::Option;
use std::str::from_utf8;

pub struct WebSocketReceiver {
	stream: TcpStream,
}

impl WebSocketReceiver {
	pub fn receive_message(&mut self) -> IoResult<WebSocketMessage> {
		let dataframe = try!(self.stream.read_websocket_dataframe());
		let mut data = dataframe.data.clone();
		match dataframe.mask {
			Some(key) => { data = mask_data(key, data.as_slice()); }
			None => { }
		}
		if !dataframe.finished {
			loop {
				let df = try!(self.stream.read_websocket_dataframe());
				match df.opcode {
					WebSocketOpcode::Continuation => {
						let mut d = df.data.clone();
						match df.mask {
							Some(key) => { d = mask_data(key, d.as_slice()); }
							None => { }
						}
						data.push_all(d.as_slice());
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

	pub fn incoming(self) -> IncomingMessages {
		IncomingMessages {
			inc: self,
		}
	}
}

pub fn new_receiver(stream: TcpStream) -> WebSocketReceiver {
	WebSocketReceiver {
		stream: stream,
	}
}

pub struct IncomingMessages {
	inc: WebSocketReceiver,
}

impl Iterator<IoResult<WebSocketMessage>> for IncomingMessages {
	fn next(&mut self) -> Option<IoResult<WebSocketMessage>> {
		Some(self.inc.receive_message())
	}
}