use super::dataframe::{WebSocketOpcode, WebSocketDataFrame, WebSocketDataFrameLength, WriteWebSocketDataFrame};
use super::mask::mask_data;
use super::WebSocketMessage;
use std::io::{Stream, IoResult, IoError, IoErrorKind};
use std::rand;
use std::clone::Clone;

/// Represents a WebSocket sender, capable of transmitting data to the remote endpoint.
/// Use the send_message() method to send a single, whole message. If you need to send
/// a series of message fragments (e.g. if you need to send a message of unknown length),
/// use the fragment() method to obtain a WebSocketFragmentSerializer, which can be
/// used to do so.
/// 
/// ```no_run
/// use websocket::message::WebSocketMessage;
/// # use websocket::WebSocketClient;
/// # use std::io::TcpStream;
/// # #[allow(unused_must_use)]
/// # fn foo() {
/// # let stream = TcpStream::connect("127.0.0.1:1234").unwrap();
/// # let mut client = WebSocketClient::new(stream, true);
/// 
/// let mut sender = client.sender(); // Get a sender
/// let data = "My fancy message".to_string();
/// let message = WebSocketMessage::Text(data.to_string());
/// 
/// let _ = sender.send_message(&message); // Send the message
/// # }
/// ```
pub struct WebSocketSender<S: Stream + Clone> {
	stream: S,
	mask: bool,
}

fn message_to_dataframe(message: &WebSocketMessage, mask: bool, finished: bool) -> WebSocketDataFrame {
	let mut opcode: WebSocketOpcode;
	let masking_key = if mask { Some(gen_key()) } else { None };
	let mut data = match *message {
		WebSocketMessage::Text(ref payload) => {				
			opcode = WebSocketOpcode::Text;
			payload.clone().into_bytes()
		}
		WebSocketMessage::Binary(ref payload) => {
			opcode = WebSocketOpcode::Binary;
			payload.clone()
		}
		WebSocketMessage::Close(ref payload) => {
			opcode = WebSocketOpcode::Close;
			payload.clone()
		}
		WebSocketMessage::Ping(ref payload) => {
			opcode = WebSocketOpcode::Ping;
			payload.clone()
		}
		WebSocketMessage::Pong(ref payload) => {
			opcode = WebSocketOpcode::Pong;
			payload.clone()
		}
	};
	
	if mask {
		data = mask_data(masking_key.unwrap(), data.as_slice());
	}
	
	let length = WebSocketDataFrameLength::new(data.len());
	
	WebSocketDataFrame {
		finished: finished,
		reserved: [false, ..3],
		opcode: opcode,
		mask: masking_key,
		length: length,
		data: data,
	}
}

impl<S: Stream + Clone> WebSocketSender<S> {
	/// Sends a message to the remote endpoint.
	pub fn send_message(&mut self, message: &WebSocketMessage) -> IoResult<()> {
		let dataframe = message_to_dataframe(message, self.mask, true);
		try!(self.stream.write_websocket_dataframe(dataframe));
		Ok(())
	}
	
	/// Returns a fragment serializer, able to send fragments of a message to the remote endpoint.
	pub fn fragment(&mut self) -> WebSocketFragmentSerializer<S> {
		WebSocketFragmentSerializer {
			inc: self,
			started: false,
		}
	}
}

/// Allows for the serialization of message fragments, to be sent to the remote endpoint.
/// Any number of messages can be sent (but only text or binary messages, as control
/// messages can never be fragmented) using the send_fragment() method, and when the final
/// fragment is to be sent, use the finish() method. After calling the finish method, the
/// remote endpoint will assemble all of the received message fragments into a single
/// message containing all of the fragment data concatenated together.
/// 
/// ```no_run
/// use websocket::message::WebSocketMessage;
/// # use websocket::WebSocketClient;
/// # use std::io::TcpStream;
/// # #[allow(unused_must_use)]
/// # fn foo() {
/// # let stream = TcpStream::connect("127.0.0.1:1234").unwrap();
/// # let mut client = WebSocketClient::new(stream, true);
/// # let mut sender = client.sender();
/// // Get a WebSocketFragmentSerializer
/// let mut fragment = sender.fragment();
/// 
/// let message1 = WebSocketMessage::Text("This ".to_string());
/// let message2 = WebSocketMessage::Text("is ".to_string());
/// let message3 = WebSocketMessage::Text("a ".to_string());
/// let message4 = WebSocketMessage::Text("fragmented ".to_string());
/// let message5 = WebSocketMessage::Text("message.".to_string());
/// 
/// // Send our fragments
/// fragment.send_fragment(&message1); // Each one is sent immediately
/// fragment.send_fragment(&message2);
/// fragment.send_fragment(&message3);
/// fragment.send_fragment(&message4);
/// 
/// // Now tell them we're done
/// fragment.finish(&message1); // Now they'll assemble the fragments into a single message
/// 
/// // Drop this WebSocketFragmentSerializer, or let it go out of scope
/// drop(fragment);
/// # }
/// ```
pub struct WebSocketFragmentSerializer<'a, S: 'a + Stream + Clone> {
	inc: &'a mut WebSocketSender<S>,
	started: bool,
}

impl<'a, S: Stream + Clone> WebSocketFragmentSerializer<'a, S> {
	/// Send a fragment of a message - if the message is finished, use the WebSocketFragmentSerializer::finish() function.
	/// Can only be used with a Text or Binary message.
	pub fn send_fragment(&mut self, message: &WebSocketMessage) -> IoResult<()> {
		let mut dataframe = message_to_dataframe(message, self.inc.mask, false);
		
		match dataframe.opcode {
			WebSocketOpcode::Text | WebSocketOpcode::Binary => {  }
			_ => {
				return Err(IoError {
					kind: IoErrorKind::InvalidInput,
					desc: "Cannot fragment a non-text/binary frame",
					detail: None,
				});
			}
		}
		
		if self.started {
			dataframe.opcode = WebSocketOpcode::Continuation;
		}
		else {
			self.started = true;
		}
		
		try!(self.inc.stream.write_websocket_dataframe(dataframe));
		Ok(())
	}
	
	/// Send the final message and tell the remote endpoint the message is complete. Can be used with an empty message if necessary.
	/// Can only be used with a Text or Binary message.
	/// 
	/// Once this has been called, you can send a new fragmented message using the send_fragment() method.
	pub fn finish(&mut self, message: &WebSocketMessage) -> IoResult<()> {
		let mut dataframe = message_to_dataframe(message, self.inc.mask, true);
		
		match dataframe.opcode {
			WebSocketOpcode::Text | WebSocketOpcode::Binary => {  }
			_ => {
				return Err(IoError {
					kind: IoErrorKind::InvalidInput,
					desc: "Cannot fragment a non-text/binary frame",
					detail: None,
				});
			}
		}
		
		if self.started {
			dataframe.opcode = WebSocketOpcode::Continuation;
			self.started = false;
		}
		
		try!(self.inc.stream.write_websocket_dataframe(dataframe));
		Ok(())
	}
}

pub fn new_sender<S: Stream + Clone>(stream: S, mask: bool) -> WebSocketSender<S> {
	WebSocketSender {
		stream: stream,
		mask: mask,
	}
}

fn gen_key() -> [u8, ..4] {
	[rand::random::<u8>(), rand::random::<u8>(), rand::random::<u8>(), rand::random::<u8>()]
}