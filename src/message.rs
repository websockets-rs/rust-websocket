use std::io::IoResult;
use result::WebSocketResult;
//use dataframe::DataFrame;

pub trait Message {
	type DataFrame;

	fn from_iter<I>(iter: I) -> WebSocketResult<Self>
		where I: Iterator<Item = WebSocketResult<<Self as Message>::DataFrame>>;
	
	fn into_iter<'a>(&'a mut self) -> &'a mut Iterator<Item = <Self as Message>::DataFrame>;
		
}

/// Represents a WebSocket message.
#[derive(PartialEq, Clone, Show)]
pub enum WebSocketMessage {
	/// A message containing UTF-8 text data
	Text(String),
	/// A message containing binary data
	Binary(Vec<u8>),
	/// A message which indicates closure of the WebSocket connection.
	/// This message may or may not contain data.
	Close(Option<CloseData>),
	/// A ping message - should be responded to with a pong message.
	/// Usually the pong message will be sent with the same data as the
	/// received ping message.
	Ping(Vec<u8>),
	/// A pong message, sent in response to a Ping message, usually
	/// containing the same data as the received ping message.
	Pong(Vec<u8>),
}

/// Represents data contained in a Close message
#[derive(PartialEq, Clone, Show)]
pub struct CloseData {
	/// The status-code of the CloseData
	pub status_code: u16,
	/// The reason-phrase of the CloseData
	pub reason: String,
}

impl CloseData {
	/// Create a new CloseData object
	pub fn new(status_code: u16, reason: String) -> CloseData {
		CloseData {
			status_code: status_code,
			reason: reason,
		}
	}
	/// Convert this into a vector of bytes
	pub fn into_bytes(self) -> IoResult<Vec<u8>> {
		let mut buf = Vec::new();
		try!(buf.write_be_u16(self.status_code));
		Ok(buf + self.reason.into_bytes().as_slice())
	}
}