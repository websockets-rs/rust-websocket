//! Utility functions for converting to Message.

use message::Message;
use dataframe::Opcode;
use result::{WebSocketResult, WebSocketError};
use std::str::from_utf8;
use byteorder::{ReadBytesExt, BigEndian};

fn bytes_to_string(data: &[u8]) -> WebSocketResult<String> {
	let utf8 = try!(from_utf8(data));
	Ok(utf8.to_string())
}
