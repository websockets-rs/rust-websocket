use super::dataframe::{WebSocketOpcode, WebSocketDataFrame, WebSocketDataFrameLength, WriteWebSocketDataFrame};
use super::mask::mask_data;
use super::WebSocketMessage;
use std::io::net::tcp::TcpStream;
use std::io::{IoResult, IoError, IoErrorKind};
use std::rand;

/// Represents a WebSocket sender, capable of transmitting data to the remote endpoint.
/// 
/// ```no_run
/// use websocket::WebSocketMessage;
/// 
/// let mut sender = client.sender(); // Get a sender
/// let data = "My fancy message".to_string();
/// let message = WebSocketMessage::Text(data.to_string());
/// 
/// let _ = sender.send_message(&message); // Send the message
/// ```
pub struct WebSocketSender {
	stream: TcpStream,
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

impl WebSocketSender {
	/// Sends a message to the remote endpoint.
	pub fn send_message(&mut self, message: &WebSocketMessage) -> IoResult<()> {
		let dataframe = message_to_dataframe(message, self.mask, true);
		try!(self.stream.write_websocket_dataframe(dataframe));
		Ok(())
	}
	
	/// Returns a fragment serializer, able to send fragments of a message to the remote endpoint.
	pub fn fragment(&mut self) -> WebSocketFragmentSerializer {
		WebSocketFragmentSerializer {
			inc: self,
			started: false,
		}
	}
}

/// Allows for the serialization of message fragments, to be sent to the remote endpoint.
/// 
/// ```no_run
/// use websocket::WebSocketMessage;
/// 
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
/// fragment.send_fragment(&message1);
/// fragment.send_fragment(&message2);
/// fragment.send_fragment(&message3);
/// fragment.send_fragment(&message4);
/// 
/// // Have to tell everyone we're done
/// fragment.finish(&message1);
/// 
/// // Drop this WebSocketFragmentSerializer
/// drop(fragment);
/// ```
pub struct WebSocketFragmentSerializer<'a> {
	inc: &'a mut WebSocketSender,
	started: bool,
}

impl<'a> WebSocketFragmentSerializer<'a> {
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

pub fn new_sender(stream: TcpStream, mask: bool) -> WebSocketSender {
	WebSocketSender {
		stream: stream,
		mask: mask,
	}
}

fn gen_key() -> [u8, ..4] {
	[rand::random::<u8>(), rand::random::<u8>(), rand::random::<u8>(), rand::random::<u8>()]
}