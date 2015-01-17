//! Utility functions for reading and writing data frame headers.

use std::io::IoResult;

bitflags! {
	/// Flags relevant to a WebSocket data frame.
	#[derive(Show)]
	flags DataFrameFlags: u8 {
		const FIN = 0x80,
		const RSV1 = 0x40,
		const RSV2 = 0x20,
		const RSV3 = 0x10,
	}
}

/// Represents a data frame header.
#[derive(Show, Copy, PartialEq)]
pub struct DataFrameHeader {
	/// The bit flags for the first byte of the header.
	pub flags: DataFrameFlags, 
	/// The opcode of the header - must be <= 16.
	pub opcode: u8, 
	/// The masking key, if any.
	pub mask: Option<u32>, 
	/// The length of the payload.
	pub len: u64
}

/// Writes a data frame header.
pub fn write_header<W>(writer: &mut W, header: DataFrameHeader) -> IoResult<()>
	where W: Writer {

	assert!(header.opcode <= 0xF);
	
	// Write 'FIN', 'RSV1', 'RSV2', 'RSV3' and 'opcode'
	try!(writer.write_u8((header.flags.bits) | header.opcode));
	
	try!(writer.write_u8(
		// Write the 'MASK'
		if header.mask.is_some() { 0x80 } else { 0x00 } |
		// Write the 'Payload len'
		if header.len <= 125 { header.len as u8 }
		else if header.len <= 65535 { 126 }
		else { 127 }
	));
	
	// Write 'Extended payload length'
	if header.len >= 126 && header.len <= 65535 {
		try!(writer.write_be_u16(header.len as u16));
	}
	else {
		try!(writer.write_be_u64(header.len));
	}
	
	// Write 'Masking-key'
	match header.mask {
		Some(mask) => try!(writer.write_be_u32(mask)),
		None => (),
	}
	
	Ok(())
}

/// Reads a data frame header.
pub fn read_header<R>(reader: &mut R) -> IoResult<DataFrameHeader>
	where R: Reader {

	let byte0 = try!(reader.read_u8());
	let byte1 = try!(reader.read_u8());
	
	let flags = DataFrameFlags::from_bits_truncate(byte0);
	let opcode = byte0 & 0x0F;
	
	let len = match byte1 & 0x7F {
		0...125 => (byte1 & 0x7F) as u64,
		126 => try!(reader.read_be_u16()) as u64,
		127 => try!(reader.read_be_u64()),
		_ => unreachable!(),
	};
	
	let mask = if byte1 & 0x80 == 0x80 {
		Some(try!(reader.read_be_u32()))
	}
	else {
		None
	};
	
	Ok(DataFrameHeader {
		flags: flags, 
		opcode: opcode, 
		mask: mask, 
		len: len
	})
}

