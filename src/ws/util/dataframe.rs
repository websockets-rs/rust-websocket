//! Utility methods for reading and writing data frames.

use std::num::FromPrimitive;

use dataframe::DataFrame;
use result::{WebSocketResult, WebSocketError};

use ws::util::header as dfh;
use ws::util::mask;

/// Writes a DataFrame to a Writer.
pub fn write_dataframe<W>(writer: &mut W, mask: bool, dataframe: DataFrame) -> WebSocketResult<()>
	where W: Writer {
	
	let mut flags = dfh::DataFrameFlags::empty();
	if dataframe.finished { flags.insert(dfh::FIN); }
	if dataframe.reserved[0] { flags.insert(dfh::RSV1); }
	if dataframe.reserved[1] { flags.insert(dfh::RSV2); }
	if dataframe.reserved[2] { flags.insert(dfh::RSV3); }
	
	let masking_key = if mask { Some(mask::gen_mask()) } else { None };
	
	let header = dfh::DataFrameHeader {
		flags: flags,
		opcode: dataframe.opcode as u8,
		mask: masking_key,
		len: dataframe.data.len() as u64,
	};
	
	try!(dfh::write_header(writer, header));
	
	match masking_key {
		Some(mask) => try!(writer.write(&mask::mask_data(mask, &dataframe.data[])[])),
		None => try!(writer.write(&dataframe.data[])),
	}
	
	Ok(())
}

/// Reads a DataFrame from a Reader.
pub fn read_dataframe<R>(reader: &mut R, should_be_masked: bool) -> WebSocketResult<DataFrame> 
	where R: Reader {

	let header = try!(dfh::read_header(reader));
	let finished = header.flags.contains(dfh::FIN);
	
	match (header.opcode, finished) {
		// Continuation opcode on first frame
		(0, _) => return Err(WebSocketError::ProtocolError(
			"Unexpected continuation data frame opcode".to_string()
		)),
		// Fragmented control frame
		(8...15, false) => return Err(WebSocketError::ProtocolError(
			"Unexpected fragmented control frame".to_string()
		)),
		_ => (),
	}
	
	Ok(DataFrame {
		finished: finished,
		reserved: [
			header.flags.contains(dfh::RSV1),
			header.flags.contains(dfh::RSV2),
			header.flags.contains(dfh::RSV3)
		],
		opcode: FromPrimitive::from_u8(header.opcode).expect("Invalid header opcode!"),
		data: match header.mask {
			Some(mask) => {
				if !should_be_masked {
					return Err(WebSocketError::DataFrameError(
						"Expected unmasked data frame".to_string()
					));
				}
				mask::mask_data(mask, &try!(reader.read_exact(header.len as usize))[])
			}
			None => {
				if should_be_masked {
					return Err(WebSocketError::DataFrameError(
						"Expected masked data frame".to_string()
					));
				}
				try!(reader.read_exact(header.len as usize))
			}
		}
	})
}

#[cfg(test)]
mod tests {
	use super::*;
	use dataframe::{DataFrame, Opcode};
	use test;
	#[test]
	fn test_read_dataframe() {
		let data = b"The quick brown fox jumps over the lazy dog";
		let mut dataframe = vec![0x81, 0x2B];
		dataframe.push_all(data);
		let obtained = read_dataframe(&mut &dataframe[], false).unwrap();
		let expected = DataFrame {
			finished: true, 
			reserved: [false; 3], 
			opcode: Opcode::Text, 
			data: data.to_vec()
		};
		assert_eq!(obtained, expected);
	}
	#[test]
	fn test_write_dataframe() {
		let data = b"The quick brown fox jumps over the lazy dog";
		let mut expected = vec![0x81, 0x2B];
		expected.push_all(data);
		let dataframe = DataFrame {
			finished: true, 
			reserved: [false; 3], 
			opcode: Opcode::Text, 
			data: data.to_vec()
		};
		let mut obtained = Vec::new();
		write_dataframe(&mut obtained, false, dataframe).unwrap();
		
		assert_eq!(&obtained[], &expected[]);
	}
	#[bench]
	fn bench_read_dataframe(b: &mut test::Bencher) {
		let data = b"The quick brown fox jumps over the lazy dog";
		let mut dataframe = vec![0x81, 0x2B];
		dataframe.push_all(data);
		b.iter(|| {
			read_dataframe(&mut &dataframe[], false).unwrap();
		});
	}
	#[bench]
	fn bench_write_dataframe(b: &mut test::Bencher) {
		let data = b"The quick brown fox jumps over the lazy dog";
		let dataframe = DataFrame {
			finished: true, 
			reserved: [false; 3], 
			opcode: Opcode::Text, 
			data: data.to_vec()
		};
		let mut writer = Vec::with_capacity(45);
		b.iter(|| {
			write_dataframe(&mut writer, false, dataframe.clone()).unwrap();
		});
	}
}