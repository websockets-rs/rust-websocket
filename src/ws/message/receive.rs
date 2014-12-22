use super::dataframe::{WebSocketOpcode, ReadWebSocketDataFrame};
use super::WebSocketMessage;
use super::mask::mask_data;
use std::iter::Iterator;
use std::io::{Stream, IoResult, IoError, IoErrorKind};
use std::option::Option;
use std::str::from_utf8;
use std::clone::Clone;

/// Represents a WebSocket receiver which can receive data from the remote endpoint.
/// All methods are task blocking (but not stream blocking, so you can send and receive concurrently).
/// A WebSocketReceiver can be captured into another task for concurrency
///
/// ```no_run
/// use websocket::message::WebSocketMessage;
/// # use websocket::{WebSocketClient, WebSocketClientMode};
/// # use std::io::TcpStream;
/// # use std::thread::Thread;
/// # #[allow(unused_must_use)]
/// # fn foo() {
/// # let stream = TcpStream::connect("127.0.0.1:1234").unwrap();
/// # let mut client = WebSocketClient::new(stream, WebSocketClientMode::RemoteServer);
/// 
/// let receiver = client.receiver();
/// 
/// Thread::spawn(move || {
/// 	// Will continuously try to receive messages
/// 	for message in receiver.incoming() {
/// 		match message {
/// 			// Match the result
/// 			Ok(message) => {
/// 				// Match the type of message
/// 				match message {
/// 					WebSocketMessage::Text(data) => {
/// 						println!("{}", data);
/// 					}
/// 					WebSocketMessage::Binary(data) => {
/// 						// ...
/// 					}
/// 					// ...
/// 					_ => { }
/// 				}
/// 			}
/// 			Err(e) => { /* Could not receive the message */ }
/// 		}
/// 	}
/// }).detach();
/// # }
/// ```
pub struct WebSocketReceiver<S: Stream + Clone> {
	stream: S,
	opcode: Option<WebSocketOpcode>,
	data: Vec<u8>,
}

impl<S: Stream + Clone> WebSocketReceiver<S> {
	/// Wait for and accept a message (subjected to the underlying stream timeout).
	/// If the received message is fragmented, this function will not return
	/// until the final fragment has been received.
	/// If a control frame is received interleaved within a fragmented message,
	/// The control frame will be returned first, and the message will be returned
	/// on the next call to the function (or later if more control frames are received).
	pub fn receive_message(&mut self) -> IoResult<WebSocketMessage> {
		let dataframe = try!(self.stream.read_websocket_dataframe());
		
		// Unmask the data if necessary
		let data = match dataframe.mask {
			Some(key) => { mask_data(key, dataframe.data.as_slice()) }
			None => { dataframe.data.clone() }
		};
		
		// Deal with the opcode type
		match dataframe.opcode {
			WebSocketOpcode::Continuation => {
				if self.opcode.is_none() {
					return Err(IoError {
						kind: IoErrorKind::InvalidInput,
						desc: "Unexpected continuation dataframe",
						detail: Some("Found a continuation dataframe, but no fragmented message received beforehand".to_string()),
					});
				}
			}
			WebSocketOpcode::Text | WebSocketOpcode::Binary => {
				if self.opcode.is_none() {
					// Set the kind - if the message is fragmented, this is the message type we'll return
					self.opcode = Some(dataframe.opcode);
				}
				else {
					return Err(IoError {
						kind: IoErrorKind::InvalidInput,
						desc: "Unexpected non-continuation dataframe",
						detail: Some("Found a text or binary frame inside a fragmented message.".to_string()),
					});
				}
			}
			// Return straight away, even if this is part of a fragment
			// TODO: Ensure the finish flag is set (although control frames
			// can never be fragmented) and the data length is zero
			WebSocketOpcode::Close => { return Ok(WebSocketMessage::Close(data)); }
			WebSocketOpcode::Ping => { return Ok(WebSocketMessage::Ping(data)); }
			WebSocketOpcode::Pong => { return Ok(WebSocketMessage::Pong(data)); }
			_ => {
				return Err(IoError {
					kind: IoErrorKind::InvalidInput,
					desc: "Unsupported dataframe opcode received",
					detail: None,
				});
			}
		}
		
		// Add the data to the buffer
		self.data.push_all(data.as_slice());
		
		if dataframe.finished {
			// We're done, so form a message
			self.create_message()
		}
		else {
			// Not done yet, so keep getting messages
			self.receive_message()
		}
	}

	fn create_message(&mut self) -> IoResult<WebSocketMessage> {
		let data = self.data.clone();
		let opcode = self.opcode.clone();
		
		self.data = Vec::new();
		self.opcode = None;
		
		match opcode {
			Some(opcode) => {
				match opcode {
					WebSocketOpcode::Text => {
						let s = try!(from_utf8(data.as_slice()).ok_or(
							IoError {
								kind: IoErrorKind::InvalidInput,
								desc: "Invalid UTF-8 sequence",
								detail: None,
							}
						));
						Ok(WebSocketMessage::Text(s.to_string()))
					}
					WebSocketOpcode::Binary => {
						Ok(WebSocketMessage::Binary(data))
					}
					_ => {
						Err(IoError {
							kind: IoErrorKind::InvalidInput,
							desc: "No opcode received!",
							detail: Some("This error should never occur. This is a bug in Rust-WebSocket".to_string()),
						})
					}
				}
			}
			None => {
				Err(IoError {
					kind: IoErrorKind::InvalidInput,
					desc: "No opcode received!",
					detail: Some("This error should never occur. This is a bug in Rust-WebSocket".to_string()),
				})
			}
		}
	}
	
	/// Returns an iterator over the incoming messages for/from this client
	pub fn incoming(self) -> IncomingMessages<S> {
		IncomingMessages {
			inc: self,
		}
	}
}

pub fn new_receiver<S: Stream + Clone>(stream: S) -> WebSocketReceiver<S> {
	WebSocketReceiver {
		stream: stream,
		opcode: None,
		data: Vec::new(),
	}
}

/// An iterator over incoming messages. Blocks the task and always returns Some.
pub struct IncomingMessages<S: Stream + Clone> {
	inc: WebSocketReceiver<S>,
}

impl<S: Stream + Clone> Iterator<IoResult<WebSocketMessage>> for IncomingMessages<S> {
	fn next(&mut self) -> Option<IoResult<WebSocketMessage>> {
		Some(self.inc.receive_message())
	}
}