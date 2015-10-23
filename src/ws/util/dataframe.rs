//! Utility methods for reading and writing data frames.

use std::io::{Read, Write};

use dataframe::{DataFrame, Opcode};
use result::{WebSocketResult, WebSocketError};

use ws::util::header as dfh;
use ws::util::mask;


/// Reads a DataFrame from a Reader.
// TODO: Move into websocket::dataframe::DataFrame impl
pub fn read_dataframe<R>(reader: &mut R, should_be_masked: bool) -> WebSocketResult<DataFrame>
	where R: Read {

	let header = try!(dfh::read_header(reader));

	Ok(DataFrame {
		finished: header.flags.contains(dfh::FIN),
		reserved: [
			header.flags.contains(dfh::RSV1),
			header.flags.contains(dfh::RSV2),
			header.flags.contains(dfh::RSV3)
		],
		opcode: Opcode::new(header.opcode).expect("Invalid header opcode!"),
		data: match header.mask {
			Some(mask) => {
				if !should_be_masked {
					return Err(WebSocketError::DataFrameError(
						"Expected unmasked data frame".to_string()
					));
				}

				let data: Vec<u8> = try!(reader.take(header.len).bytes().collect());
				mask::mask_data(mask, &data)
			}
			None => {
				if should_be_masked {
					return Err(WebSocketError::DataFrameError(
						"Expected masked data frame".to_string()
					));
				}

				try!(reader.take(header.len).bytes().collect())
			}
		}
	})
}

#[cfg(all(feature = "nightly", test))]
mod tests {
	use super::*;
	use dataframe::{DataFrame, Opcode};
	use test;
	#[test]
	fn test_read_dataframe() {
		let data = b"The quick brown fox jumps over the lazy dog";
		let mut dataframe = vec![0x81, 0x2B];
		for i in data.iter() {
			dataframe.push(*i);
		}
		let obtained = read_dataframe(&mut &dataframe[..], false).unwrap();
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
		for i in data.iter() {
			expected.push(*i);
		}
		let dataframe = DataFrame {
			finished: true,
			reserved: [false; 3],
			opcode: Opcode::Text,
			data: data.to_vec()
		};
		let mut obtained = Vec::new();
		write_dataframe(&mut obtained, false, dataframe).unwrap();

		assert_eq!(&obtained[..], &expected[..]);
	}
	#[bench]
	fn bench_read_dataframe(b: &mut test::Bencher) {
		let data = b"The quick brown fox jumps over the lazy dog";
		let mut dataframe = vec![0x81, 0x2B];
		for i in data.iter() {
			dataframe.push(*i);
		}
		b.iter(|| {
			read_dataframe(&mut &dataframe[..], false).unwrap();
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
