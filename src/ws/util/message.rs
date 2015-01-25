//! Utility functions for converting to Message.

use message::{Message, CloseData};
use dataframe::Opcode;
use result::{WebSocketResult, WebSocketError};
use std::str::from_utf8;

/// Creates a Message from an Opcode and data.
pub fn message_from_data(opcode: Opcode, data: Vec<u8>) -> WebSocketResult<Message> {
	Ok(match opcode {	
		Opcode::Text => Message::Text(try!(bytes_to_string(&data[]))),
		Opcode::Binary => Message::Binary(data),
		Opcode::Close => {
			if data.len() > 0 {				
				let status_code = try!((&data[]).read_be_u16());
				let reason = try!(bytes_to_string(&data[2..]));
				let close_data = CloseData::new(status_code, reason);
				Message::Close(Some(close_data))
			}
			else {
				Message::Close(None)
			}
		}
		Opcode::Ping => Message::Ping(data),
		Opcode::Pong => Message::Pong(data),
		_ => return Err(WebSocketError::ProtocolError(
			"Unsupported opcode received".to_string()
		)),
	})
}

fn bytes_to_string(data: &[u8]) -> WebSocketResult<String> {
	let utf8 = try!(from_utf8(data));
	Ok(utf8.to_string())
}