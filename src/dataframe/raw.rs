//! Provides a way to deal with data frames at a lower level
#![stable]
use dataframe::opcode::WebSocketOpcode;
use common::{WebSocketResult, WebSocketError};

/// Represents the complete structure of a WebSocket data frame
/// 
/// Unlike a WebSocketDataFrame, the data in a RawDataFrame is exactly as it was
/// constructed, i.e. it may or may not be masked.
#[derive(Clone, Show, PartialEq)]
#[stable]
pub struct RawDataFrame {
	/// The FIN bit
	pub finished: bool,
	/// The reserved bits
	pub reserved: [bool; 3],
	/// The opcode
	pub opcode: WebSocketOpcode,
	/// The masking key, if any
	pub mask: Option<[u8; 4]>,
	/// The payload length
	pub length: DataFrameLength,
	/// THe payload
	pub data: Vec<u8>,
}

#[stable]
impl RawDataFrame {
	/// Helper function that reads a RawDataFrame from the Reader.
	#[stable]
	pub fn read<R: Reader>(reader: &mut R) -> WebSocketResult<RawDataFrame> {
		let byte0 = try!(reader.read_byte());
		let finished = (byte0 >> 7) & 0x01 == 0x01;
		let reserved = [
			(byte0 >> 6) & 0x01 == 0x01,
			(byte0 >> 5) & 0x01 == 0x01,
			(byte0 >> 4) & 0x01 == 0x01
		];
		let opcode_nibble = byte0 & 0x0F;
		let opcode = match opcode_nibble {
			0x0 => { WebSocketOpcode::Continuation }
			0x1 => { WebSocketOpcode::Text }
			0x2 => { WebSocketOpcode::Binary }
			0x3 => { WebSocketOpcode::NonControl1 }
			0x4 => { WebSocketOpcode::NonControl2 }
			0x5 => { WebSocketOpcode::NonControl3 }
			0x6 => { WebSocketOpcode::NonControl4 }
			0x7 => { WebSocketOpcode::NonControl5 }
			0x8 => { WebSocketOpcode::Close }
			0x9 => { WebSocketOpcode::Ping }
			0xA => { WebSocketOpcode::Pong }
			0xB => { WebSocketOpcode::Control1 }
			0xC => { WebSocketOpcode::Control2 }
			0xD => { WebSocketOpcode::Control3 }
			0xE => { WebSocketOpcode::Control4 }
			0xF => { WebSocketOpcode::Control5 }
			_ => { return Err(WebSocketError::ProtocolError("Invalid WebSocket opcode".to_string())); }
		};
		
		let byte1 = try!(reader.read_byte());
		let mask = (byte1 >> 7) & 0x01 == 0x01;
		let length_tiny = byte1 & 0x7F;
		
		let mut length = DataFrameLength::Tiny(length_tiny);
		
		if length_tiny == 126 {
			let length_short = try!(reader.read_be_u16());
			length = DataFrameLength::Short(length_short);
		}
		else if length_tiny == 127 {
			let length_long = try!(reader.read_be_u64());
			length = DataFrameLength::Long(length_long);
		}
		
		let masking_key: Option<[u8; 4]> =
			if mask {
				Some([
					try!(reader.read_byte()),
					try!(reader.read_byte()),
					try!(reader.read_byte()),
					try!(reader.read_byte())
				])
			}
			else {
				None
			};
		
		let data = try!(reader.read_exact(length.unwrap()));
		
		Ok(RawDataFrame {
			finished: finished,
			reserved: reserved,
			opcode: opcode,
			mask: masking_key,
			length: length,
			data: data,
		})
	}

	/// Helper function that writes a RawDataFrame to the Writer.
	#[stable]
	pub fn write<W: Writer>(&self, writer: &mut W) -> WebSocketResult<()> {
		let mut byte0: u8 = 0x00;
		if self.finished { byte0 |= 0x80; }
		if self.reserved[0] { byte0|= 0x40; }
		if self.reserved[1] { byte0|= 0x20; }
		if self.reserved[2] { byte0|= 0x10; }
		byte0 |= match self.opcode {
			WebSocketOpcode::Continuation => { 0x0 }
			WebSocketOpcode::Text => { 0x1 }
			WebSocketOpcode::Binary => { 0x2 }
			WebSocketOpcode::NonControl1 => { 0x3 }
			WebSocketOpcode::NonControl2 => { 0x4 }
			WebSocketOpcode::NonControl3 => { 0x5 }
			WebSocketOpcode::NonControl4 => { 0x6 }
			WebSocketOpcode::NonControl5 => { 0x7 }
			WebSocketOpcode::Close => { 0x8 }
			WebSocketOpcode::Ping => { 0x9 }
			WebSocketOpcode::Pong => { 0xA }
			WebSocketOpcode::Control1 => { 0xB }
			WebSocketOpcode::Control2 => { 0xC }
			WebSocketOpcode::Control3 => { 0xD }
			WebSocketOpcode::Control4 => { 0xE }
			WebSocketOpcode::Control5 => { 0xF }
		};
		
		try!(writer.write_u8(byte0));
		
		let mut byte1: u8 = 0x00;
		
		if self.mask.is_some() { byte1 |= 0x80; }
		
		match self.length {
			DataFrameLength::Tiny(length) => {
				byte1 |= length;
				try!(writer.write_u8(byte1));
			}
			DataFrameLength::Short(length) => {
				byte1 |= 0x7E;
				try!(writer.write_u8(byte1));
				try!(writer.write_be_u16(length));
			}
			DataFrameLength::Long(length) => {
				byte1 |= 0x7F;
				try!(writer.write_u8(byte1));
				try!(writer.write_be_u64(length));
			}
		}
		
		match self.mask {
			Some(key) => { try!(writer.write(key.as_slice())); }
			None => { }
		}
		
		try!(writer.write(self.data.as_slice()));
		
		Ok(())
	}
}

/// Represents a payload length, which can be either 8, 16 or 64 bits long
#[derive(Clone, Show, Copy, PartialEq)]
#[stable]
pub enum DataFrameLength {
	/// Data frame lengths less than 126
	Tiny(u8),
	/// Data frame lengths between 126 and 65535
	Short(u16),
	/// Data frame lengths above 65535
	Long(u64),
}

#[stable]
impl DataFrameLength {
	/// Retrieve the underlying length as a uint
	#[stable]
	pub fn unwrap(self) -> uint {
		match self {
			DataFrameLength::Tiny(length) => { length as uint }
			DataFrameLength::Short(length) => { length as uint }
			DataFrameLength::Long(length) => { length as uint }
		}
	}
	/// Create a new DataFrameLength
	#[stable]
	pub fn new(length: uint) -> DataFrameLength {
		if length <= 125 {
			DataFrameLength::Tiny(length as u8)
		}
		else if length <= 65535 {
			DataFrameLength::Short(length as u16)
		}
		else {
			DataFrameLength::Long(length as u64)
		}
	}
}

#[test]
fn test_read_dataframe() {
	let mut buffer: &[u8] = &[0x81, 0x06, 0x00, 0xFF, 0x0F, 0xF0, 0x33, 0xCC];
	let dataframe = RawDataFrame::read(&mut buffer).unwrap();
	let expected = RawDataFrame {
		finished: true,
		reserved: [false; 3],
		opcode: WebSocketOpcode::Text,
		mask: None,
		length: DataFrameLength::Tiny(6),
		data: vec![0x00, 0xFF, 0x0F, 0xF0, 0x33, 0xCC],
	};
	
	assert_eq!(dataframe, expected);
}

#[test]
fn test_write_dataframe() {
	let dataframe = RawDataFrame {
		finished: true,
		reserved: [false; 3],
		opcode: WebSocketOpcode::Text,
		mask: None,
		length: DataFrameLength::Tiny(6),
		data: vec![0x00, 0xFF, 0x0F, 0xF0, 0x33, 0xCC],
	};
	let mut buffer = Vec::new();
	dataframe.write(&mut buffer).unwrap();
	let expected: &[u8] = &[0x81, 0x06, 0x00, 0xFF, 0x0F, 0xF0, 0x33, 0xCC];
	assert_eq!(buffer, expected);
}