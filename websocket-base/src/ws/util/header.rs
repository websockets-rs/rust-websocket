//! Utility functions for reading and writing data frame headers.

use crate::result::{WebSocketError, WebSocketResult};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

bitflags! {
	/// Flags relevant to a WebSocket data frame.
	pub struct DataFrameFlags: u8 {
		/// Marks this dataframe as the last dataframe
		const FIN = 0x80;
		/// First reserved bit
		const RSV1 = 0x40;
		/// Second reserved bit
		const RSV2 = 0x20;
		/// Third reserved bit
		const RSV3 = 0x10;
	}
}

/// Represents a data frame header.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DataFrameHeader {
	/// The bit flags for the first byte of the header.
	pub flags: DataFrameFlags,
	/// The opcode of the header - must be <= 16.
	pub opcode: u8,
	/// The masking key, if any.
	pub mask: Option<[u8; 4]>,
	/// The length of the payload.
	pub len: u64,
}

/// Writes a data frame header.
pub fn write_header(writer: &mut dyn Write, header: DataFrameHeader) -> WebSocketResult<()> {
	if header.opcode > 0xF {
		return Err(WebSocketError::DataFrameError("Invalid data frame opcode"));
	}
	if header.opcode >= 8 && header.len >= 126 {
		return Err(WebSocketError::DataFrameError(
			"Control frame length too long",
		));
	}

	// Write 'FIN', 'RSV1', 'RSV2', 'RSV3' and 'opcode'
	writer.write_u8((header.flags.bits) | header.opcode)?;

	writer.write_u8(
		// Write the 'MASK'
		if header.mask.is_some() { 0x80 } else { 0x00 } |
		// Write the 'Payload len'
		if header.len <= 125 { header.len as u8 }
		else if header.len <= 65535 { 126 }
		else { 127 },
	)?;

	// Write 'Extended payload length'
	if header.len >= 126 && header.len <= 65535 {
		writer.write_u16::<BigEndian>(header.len as u16)?;
	} else if header.len > 65535 {
		writer.write_u64::<BigEndian>(header.len)?;
	}

	// Write 'Masking-key'
	if let Some(mask) = header.mask {
		writer.write_all(&mask)?
	}

	Ok(())
}

/// Reads a data frame header.
pub fn read_header<R>(reader: &mut R) -> WebSocketResult<DataFrameHeader>
where
	R: Read,
{
	let byte0 = reader.read_u8()?;
	let byte1 = reader.read_u8()?;

	let flags = DataFrameFlags::from_bits_truncate(byte0);
	let opcode = byte0 & 0x0F;

	let len = match byte1 & 0x7F {
		0..=125 => u64::from(byte1 & 0x7F),
		126 => {
			let len = u64::from(reader.read_u16::<BigEndian>()?);
			if len <= 125 {
				return Err(WebSocketError::DataFrameError("Invalid data frame length"));
			}
			len
		}
		127 => {
			let len = reader.read_u64::<BigEndian>()?;
			if len <= 65535 {
				return Err(WebSocketError::DataFrameError("Invalid data frame length"));
			}
			len
		}
		_ => unreachable!(),
	};

	if opcode >= 8 {
		if len >= 126 {
			return Err(WebSocketError::DataFrameError(
				"Control frame length too long",
			));
		}
		if !flags.contains(DataFrameFlags::FIN) {
			return Err(WebSocketError::ProtocolError(
				"Illegal fragmented control frame",
			));
		}
	}

	let mask = if byte1 & 0x80 == 0x80 {
		Some([
			reader.read_u8()?,
			reader.read_u8()?,
			reader.read_u8()?,
			reader.read_u8()?,
		])
	} else {
		None
	};

	Ok(DataFrameHeader {
		flags,
		opcode,
		mask,
		len,
	})
}

#[cfg(all(feature = "nightly", test))]
mod tests {
	use super::*;
	use test;

	#[test]
	fn test_read_header_simple() {
		let header = [0x81, 0x2B];
		let obtained = read_header(&mut &header[..]).unwrap();
		let expected = DataFrameHeader {
			flags: DataFrameFlags::FIN,
			opcode: 1,
			mask: None,
			len: 43,
		};
		assert_eq!(obtained, expected);
	}

	#[test]
	fn test_write_header_simple() {
		let header = DataFrameHeader {
			flags: DataFrameFlags::FIN,
			opcode: 1,
			mask: None,
			len: 43,
		};
		let expected = [0x81, 0x2B];
		let mut obtained = Vec::with_capacity(2);
		write_header(&mut obtained, header).unwrap();

		assert_eq!(&obtained[..], &expected[..]);
	}

	#[test]
	fn test_read_header_complex() {
		let header = [0x42, 0xFE, 0x02, 0x00, 0x02, 0x04, 0x08, 0x10];
		let obtained = read_header(&mut &header[..]).unwrap();
		let expected = DataFrameHeader {
			flags: DataFrameFlags::RSV1,
			opcode: 2,
			mask: Some([2, 4, 8, 16]),
			len: 512,
		};
		assert_eq!(obtained, expected);
	}

	#[test]
	fn test_write_header_complex() {
		let header = DataFrameHeader {
			flags: DataFrameFlags::RSV1,
			opcode: 2,
			mask: Some([2, 4, 8, 16]),
			len: 512,
		};
		let expected = [0x42, 0xFE, 0x02, 0x00, 0x02, 0x04, 0x08, 0x10];
		let mut obtained = Vec::with_capacity(8);
		write_header(&mut obtained, header).unwrap();

		assert_eq!(&obtained[..], &expected[..]);
	}

	#[bench]
	fn bench_read_header(b: &mut test::Bencher) {
		let header = vec![0x42u8, 0xFE, 0x02, 0x00, 0x02, 0x04, 0x08, 0x10];
		b.iter(|| {
			read_header(&mut &header[..]).unwrap();
		});
	}

	#[bench]
	fn bench_write_header(b: &mut test::Bencher) {
		let header = DataFrameHeader {
			flags: DataFrameFlags::RSV1,
			opcode: 2,
			mask: Some([2, 4, 8, 16]),
			len: 512,
		};
		let mut writer = Vec::with_capacity(8);
		b.iter(|| {
			write_header(&mut writer, header).unwrap();
		});
	}
}
