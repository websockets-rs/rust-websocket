//! Structs for dealing with WebSocket data frames
#[unstable]
pub use self::opcode::WebSocketOpcode;

pub use self::sender::{WebSocketDataFrameSender, WebSocketSender};
pub use self::receiver::{WebSocketDataFrameReceiver, WebSocketReceiver};
pub use self::converter::{WebSocketDataFrameConverter, WebSocketConverter};

pub mod mask;
pub mod opcode;
pub mod raw;
pub mod converter;
pub mod sender;
pub mod receiver;

/// Represents a WebSocket data frame. The data held in a WebSocketDataFrame is never masked.
#[deriving(Clone, Send, PartialEq)]
#[stable]
pub struct WebSocketDataFrame {
	/// Whether or no this constitutes the end of a message
	pub finished: bool,
	/// The reserved portion of the data frame (RFC6455 5.2)
	pub reserved: [bool, ..3],
	/// The opcode associated with this data frame
	pub opcode: WebSocketOpcode,
	/// The payload associated with this data frame
	pub data: Vec<u8>,
}

impl WebSocketDataFrame {
	/// Creates a new WebSocketDataFrame.
	#[stable]
	pub fn new(finished: bool, opcode: WebSocketOpcode, data: Vec<u8>) -> WebSocketDataFrame {
		WebSocketDataFrame {
			finished: finished,
			reserved: [false, ..3],
			opcode: opcode,
			data: data,
		}
	}
}