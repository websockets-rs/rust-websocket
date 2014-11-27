use std::option::Option;
use std::io::{Reader, Writer, IoResult, IoError, IoErrorKind};

pub struct WebSocketDataFrame {
	pub finished: bool,
	pub reserved: [bool, ..3],
	pub opcode: WebSocketOpcode,
	pub mask: Option<[u8, ..4]>,
	pub length: WebSocketDataFrameLength,
	pub data: Vec<u8>,
}

pub enum WebSocketOpcode {
	Continuation,
	Text,
	Binary,
	NonControl1,
	NonControl2,
	NonControl3,
	NonControl4,
	NonControl5,
	Close,
	Ping,
	Pong,
	Control1,
	Control2,
	Control3,
	Control4,
	Control5,
}

pub enum WebSocketDataFrameLength {
	Tiny(u8),
	Short(u16),
	Long(u64),
}

impl WebSocketDataFrameLength {
	pub fn unwrap(self) -> uint {
		match self {
			WebSocketDataFrameLength::Tiny(length) => { length as uint }
			WebSocketDataFrameLength::Short(length) => { length as uint }
			WebSocketDataFrameLength::Long(length) => { length as uint }
		}
	}
	pub fn new(length: uint) -> WebSocketDataFrameLength {
		if length <= 125 {
			WebSocketDataFrameLength::Tiny(length as u8)
		}
		else if length <= 65535 {
			WebSocketDataFrameLength::Short(length as u16)
		}
		else {
			WebSocketDataFrameLength::Long(length as u64)
		}
	}
}

pub trait ReadWebSocketDataFrame {
	fn read_websocket_dataframe(&mut self) -> IoResult<WebSocketDataFrame>;
}

impl<R: Reader> ReadWebSocketDataFrame for R {
	fn read_websocket_dataframe(&mut self) -> IoResult<WebSocketDataFrame> {
		let byte0 = try!(self.read_byte());
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
			_ => {
				return Err(IoError {
					kind: IoErrorKind::InvalidInput,
					desc: "Invalid opcode received",
					detail: None,
				});
			}
		};
		
		let byte1 = try!(self.read_byte());
		let mask = (byte1 >> 7) & 0x01 == 0x01;
		let length_tiny = byte1 & 0x7F;
		
		let mut length = WebSocketDataFrameLength::Tiny(length_tiny);
		
		if length_tiny == 126 {
			let length_short = try!(self.read_be_u16());
			length = WebSocketDataFrameLength::Short(length_short);
		}
		else if length_tiny == 127 {
			let length_long = try!(self.read_be_u64());
			length = WebSocketDataFrameLength::Long(length_long);
		}
		
		let mut masking_key: Option<[u8, ..4]> = None;
		if mask {
			masking_key = Some([
				try!(self.read_byte()),
				try!(self.read_byte()),
				try!(self.read_byte()),
				try!(self.read_byte())
			]);
		}
		
		let data = try!(self.read_exact(length.unwrap()));
		
		Ok(WebSocketDataFrame {
			finished: finished,
			reserved: reserved,
			opcode: opcode,
			mask: masking_key,
			length: length,
			data: data,
		})
	}
}

pub trait WriteWebSocketDataFrame {
	fn write_websocket_dataframe(&mut self, dataframe: WebSocketDataFrame) -> IoResult<()>;
}

impl <W: Writer> WriteWebSocketDataFrame for W {
	fn write_websocket_dataframe(&mut self, dataframe: WebSocketDataFrame) -> IoResult<()> {
		let mut byte0: u8 = 0x00;
		if dataframe.finished { byte0 |= 0x80; }
		if dataframe.reserved[0] { byte0|= 0x40; }
		if dataframe.reserved[1] { byte0|= 0x20; }
		if dataframe.reserved[2] { byte0|= 0x10; }
		byte0 |= match dataframe.opcode {
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
		
		try!(self.write_u8(byte0));
		
		let mut byte1: u8 = 0x00;
		
		if dataframe.mask.is_some() { byte1 |= 0x80; }
		
		match dataframe.length {
			WebSocketDataFrameLength::Tiny(length) => {
				byte1 |= length;
				try!(self.write_u8(byte1));
			}
			WebSocketDataFrameLength::Short(length) => {
				byte1 |= 0x7E;
				try!(self.write_u8(byte1));
				try!(self.write_be_u16(length));
			}
			WebSocketDataFrameLength::Long(length) => {
				byte1 |= 0x7F;
				try!(self.write_u8(byte1));
				try!(self.write_be_u64(length));
			}
		}
		
		match dataframe.mask {
			Some(key) => { try!(self.write(key.as_slice())); }
			None => { }
		}
		
		try!(self.write(dataframe.data.as_slice()));
		
		Ok(())
	}
}