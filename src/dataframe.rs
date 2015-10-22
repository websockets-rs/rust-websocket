//! Module containing the default implementation of data frames.
use std::io::Write;
use std::io::Result as IoResult;
use ws;

/// Represents a WebSocket data frame.
///
/// The data held in a DataFrame is never masked.
/// Masking/unmasking is done when sending and receiving the data frame,
#[derive(Debug, Clone, PartialEq)]
pub struct DataFrame {
	/// Whether or no this constitutes the end of a message
	pub finished: bool,
	/// The reserved portion of the data frame (RFC6455 5.2)
	pub reserved: [bool; 3],
	/// The opcode associated with this data frame
	pub opcode: Opcode,
	/// The payload associated with this data frame
	pub data: Vec<u8>,
}

impl DataFrame {
	/// Creates a new DataFrame.
	pub fn new(finished: bool, opcode: Opcode, data: Vec<u8>) -> DataFrame {
		DataFrame {
			finished: finished,
			reserved: [false; 3],
			opcode: opcode,
			data: data,
		}
	}
}

impl ws::dataframe::DataFrame for DataFrame {
    fn is_last(&self) -> bool {
		self.finished
	}

    fn opcode(&self) -> Opcode {
		self.opcode
	}

    fn reserved<'a>(&'a self) -> &'a [bool; 3] {
		&self.reserved
	}

	fn payload<'a>(&'a self) -> &'a [u8] {
		&self.data
	}

    fn write_payload<W>(&self, socket: &mut W) -> IoResult<()>
    where W: Write {
		unimplemented!();
	}
}

pub struct DataFrameRef<'a> {
	pub finished: bool,
	pub reserved: [bool; 3],
	pub opcode: Opcode,
	pub data: &'a [u8],
}

impl<'a> ws::dataframe::DataFrame for DataFrameRef<'a> {
    fn is_last(&self) -> bool {
		self.finished
	}

    fn opcode(&self) -> Opcode {
		self.opcode
	}

    fn reserved<'b>(&'b self) -> &'b [bool; 3] {
		&self.reserved
	}

	fn payload<'b>(&'b self) -> &'b [u8] {
		&self.data
	}

    fn write_payload<W>(&self, socket: &mut W) -> IoResult<()>
    where W: Write {
		unimplemented!();
	}
}

/// Represents a WebSocket data frame opcode
#[derive(Clone, Debug, Copy, PartialEq)]
pub enum Opcode {
	/// A continuation data frame
	Continuation,
	/// A UTF-8 text data frame
	Text,
	/// A binary data frame
	Binary,
	/// An undefined non-control data frame
	NonControl1,
	/// An undefined non-control data frame
	NonControl2,
	/// An undefined non-control data frame
	NonControl3,
	/// An undefined non-control data frame
	NonControl4,
	/// An undefined non-control data frame
	NonControl5,
	/// A close data frame
	Close,
	/// A ping data frame
	Ping,
	/// A pong data frame
	Pong,
	/// An undefined control data frame
	Control1,
	/// An undefined control data frame
	Control2,
	/// An undefined control data frame
	Control3,
	/// An undefined control data frame
	Control4,
	/// An undefined control data frame
	Control5,
}

impl Opcode {
	/// Attempts to form an Opcode from a nibble.
	///
	/// Returns the Opcode, or None if the opcode is out of range.
	pub fn new(op: u8) -> Option<Opcode> {
		Some(match op {
			0 => Opcode::Continuation,
			1 => Opcode::Text,
			2 => Opcode::Binary,
			3 => Opcode::NonControl1,
			4 => Opcode::NonControl2,
			5 => Opcode::NonControl3,
			6 => Opcode::NonControl4,
			7 => Opcode::NonControl5,
			8 => Opcode::Close,
			9 => Opcode::Ping,
			10 => Opcode::Pong,
			11 => Opcode::Control1,
			12 => Opcode::Control2,
			13 => Opcode::Control3,
			14 => Opcode::Control4,
			15 => Opcode::Control5,
			_ => return None,
		})
	}
}
