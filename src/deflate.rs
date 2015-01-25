//! Module containing the permessage-deflate backed message type.

use message::Message;
use std::iter::{Take, Repeat};
use result::{WebSocketResult, WebSocketError};
use dataframe::{DataFrame, Opcode};
use ws::util::message::message_from_data;
use ws;

use flate2::reader::DeflateDecoder;
use flate2::reader::DeflateEncoder;

/// Represents a permessage-deflate capable WebSocket message.
#[derive(PartialEq, Clone, Debug)]
pub struct DeflateMessage(Message);

impl ws::Message<DataFrame> for DeflateMessage {
	type DataFrameIterator = Take<Repeat<DataFrame>>;
	/// Attempt to form a message from a series of data frames
	fn from_dataframes(mut frames: Vec<DataFrame>) -> WebSocketResult<DeflateMessage> {
		if frames.len() == 0 {
			return Err(WebSocketError::ProtocolError(
				"No dataframes provided".to_string()
			));
		}
		
		let first = frames[0].clone();
		
		if first.reserved == [true, false, false] && (first.opcode as u8) < 8 {
			frames[0].reserved = [false; 3];
			frames[0].opcode = Opcode::Binary;
			let message = try!(ws::Message::from_dataframes(frames));
			if let Message::Binary(data) = message {
				
				Ok(DeflateMessage(
					try!(message_from_data(first.opcode, data))
				))
			}
			else {
				unreachable!();
			}
		}
		else {
			let message = try!(ws::Message::from_dataframes(frames));
			Ok(DeflateMessage(message))
		}
	}
	/// Turns this message into an iterator over data frames
	fn into_iter(self) -> Take<Repeat<DataFrame>> {

	}
}
