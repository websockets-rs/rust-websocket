use std::num::FromPrimitive;
use std::io::IoResult;

use common::WebSocketDataFrame;
use result::{WebSocketResult, WebSocketError};

use ws::util::header as dfh;
use ws::util::mask;

/// Writes a WebSocketDataFrame to a Writer.
pub fn write_dataframe<W>(writer: &mut W, mask: bool, dataframe: WebSocketDataFrame) -> WebSocketResult<()>
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

/// Reads a WebSocketDataFrame from a Reader.
pub fn read_dataframe<R>(reader: &mut R, should_be_masked: bool) -> WebSocketResult<WebSocketDataFrame> 
	where R: Reader {

	let header = try!(dfh::read_header(reader));
	
	Ok(WebSocketDataFrame {
		finished: header.flags.contains(dfh::FIN),
		reserved: [
			header.flags.contains(dfh::RSV1),
			header.flags.contains(dfh::RSV2),
			header.flags.contains(dfh::RSV3)
		],
		opcode: FromPrimitive::from_u8(header.opcode).expect("Invalid header opcode!"),
		data: match header.mask {
			Some(mask) => {
				if !should_be_masked {
					return Err(WebSocketError::ProtocolError(
						"Expected unmasked data frame".to_string()
					));
				}
				mask::mask_data(mask, &try!(reader.read_exact(header.len as usize))[])
			}
			None => {
				if should_be_masked {
					return Err(WebSocketError::ProtocolError(
						"Expected masked data frame".to_string()
					));
				}
				try!(reader.read_exact(header.len as usize))
			}
		}
	})
}

