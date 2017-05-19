//! Module containing the default implementation of data frames.
use std::io::Read;
use std::borrow::Cow;
use result::{WebSocketResult, WebSocketError};
use ws::dataframe::DataFrame as DataFrameable;
use ws::util::header as dfh;
use ws::util::mask;

/// Represents a WebSocket data frame.
///
/// The data held in a DataFrame is never masked.
/// Masking/unmasking is done when sending and receiving the data frame,
///
/// This DataFrame, unlike the standard Message implementation (which also
/// implements the DataFrame trait), owns its entire payload. This means that calls to `payload`
/// don't allocate extra memory (again unlike the default Message implementation).
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

	/// Reads a DataFrame from a Reader.
	pub fn read_dataframe<R>(reader: &mut R, should_be_masked: bool) -> WebSocketResult<Self>
		where R: Read
	{
		let header = try!(dfh::read_header(reader));

		Ok(DataFrame {
		       finished: header.flags.contains(dfh::FIN),
		       reserved: [
			header.flags.contains(dfh::RSV1),
			header.flags.contains(dfh::RSV2),
			header.flags.contains(dfh::RSV3),
		],
		       opcode: Opcode::new(header.opcode).expect("Invalid header opcode!"),
		       data: match header.mask {
		           Some(mask) => {
			if !should_be_masked {
				return Err(WebSocketError::DataFrameError("Expected unmasked data frame"));
			}
			let mut data: Vec<u8> = Vec::with_capacity(header.len as usize);
			try!(reader.take(header.len).read_to_end(&mut data));
			mask::mask_data(mask, &data)
		}
		           None => {
			if should_be_masked {
				return Err(WebSocketError::DataFrameError("Expected masked data frame"));
			}
			let mut data: Vec<u8> = Vec::with_capacity(header.len as usize);
			try!(reader.take(header.len).read_to_end(&mut data));
			data
		}
		       },
		   })
	}
}

impl DataFrameable for DataFrame {
	#[inline(always)]
	fn is_last(&self) -> bool {
		self.finished
	}

	#[inline(always)]
	fn opcode(&self) -> u8 {
		self.opcode as u8
	}

	#[inline(always)]
	fn reserved(&self) -> &[bool; 3] {
		&self.reserved
	}

	#[inline(always)]
	fn payload(&self) -> Cow<[u8]> {
		Cow::Borrowed(&self.data)
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

#[cfg(all(feature = "nightly", test))]
mod tests {
	use super::*;
	use ws::dataframe::DataFrame as DataFrameable;
	use test::Bencher;

	#[test]
	fn test_read_dataframe() {
		let data = b"The quick brown fox jumps over the lazy dog";
		let mut dataframe = vec![0x81, 0x2B];
		for i in data.iter() {
			dataframe.push(*i);
		}
		let obtained = DataFrame::read_dataframe(&mut &dataframe[..], false).unwrap();
		let expected = DataFrame {
			finished: true,
			reserved: [false; 3],
			opcode: Opcode::Text,
			data: data.to_vec(),
		};
		assert_eq!(obtained, expected);
	}
	#[bench]
	fn bench_read_dataframe(b: &mut Bencher) {
		let data = b"The quick brown fox jumps over the lazy dog";
		let mut dataframe = vec![0x81, 0x2B];
		for i in data.iter() {
			dataframe.push(*i);
		}
		b.iter(|| { DataFrame::read_dataframe(&mut &dataframe[..], false).unwrap(); });
	}

	#[test]
	fn test_write_dataframe() {
		let data = b"The quick brown fox jumps over the lazy dog";
		let mut expected = vec![0x81, 0x2B];
		for i in data.iter() {
			expected.push(*i);
		}
		let dataframe = DataFrame {
			finished: true,
			reserved: [false; 3],
			opcode: Opcode::Text,
			data: data.to_vec(),
		};
		let mut obtained = Vec::new();
		dataframe.write_to(&mut obtained, false).unwrap();

		assert_eq!(&obtained[..], &expected[..]);
	}

	#[bench]
	fn bench_write_dataframe(b: &mut Bencher) {
		let data = b"The quick brown fox jumps over the lazy dog";
		let dataframe = DataFrame {
			finished: true,
			reserved: [false; 3],
			opcode: Opcode::Text,
			data: data.to_vec(),
		};
		let mut writer = Vec::with_capacity(45);
		b.iter(|| { dataframe.write_to(&mut writer, false).unwrap(); });
	}
}
