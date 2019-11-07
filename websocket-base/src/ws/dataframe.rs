//! Describes the generic DataFrame, defining a trait
//! that all dataframes should share. This is so one can
//! optimize the memory footprint of a dataframe for their
//! own needs, and be able to use custom dataframes quickly
use crate::result::WebSocketResult;
use crate::ws::util::header as dfh;
use crate::ws::util::mask;
use crate::ws::util::mask::Masker;
use std::io::Write;

/// A generic DataFrame. Every dataframe should be able to
/// provide these methods. (If the payload is not known in advance then
/// rewrite the write_payload method)
pub trait DataFrame {
	/// Is this dataframe the final dataframe of the message?
	fn is_last(&self) -> bool;
	/// What type of data does this dataframe contain?
	fn opcode(&self) -> u8;
	/// Reserved bits of this dataframe
	fn reserved(&self) -> &[bool; 3];

	/// How long (in bytes) is this dataframe's payload
	fn size(&self) -> usize;

	/// Get's the size of the entire dataframe in bytes,
	/// i.e. header and payload.
	fn frame_size(&self, masked: bool) -> usize {
		// one byte for the opcode & reserved & fin
		1
        // depending on the size of the payload, add the right payload len bytes
        + match self.size() {
            s if s <= 125 => 1,
            s if s <= 65535 => 3,
            _ => 9,
        }
        // add the mask size if there is one
        + if masked {
            4
        } else {
            0
        }
        // finally add the payload len
        + self.size()
	}

	/// Write the payload to a writer
	fn write_payload(&self, socket: &mut dyn Write) -> WebSocketResult<()>;

	/// Takes the payload out into a vec
	fn take_payload(self) -> Vec<u8>;

	/// Writes a DataFrame to a Writer.
	fn write_to(&self, writer: &mut dyn Write, mask: bool) -> WebSocketResult<()> {
		let mut flags = dfh::DataFrameFlags::empty();
		if self.is_last() {
			flags.insert(dfh::DataFrameFlags::FIN);
		}
		{
			let reserved = self.reserved();
			if reserved[0] {
				flags.insert(dfh::DataFrameFlags::RSV1);
			}
			if reserved[1] {
				flags.insert(dfh::DataFrameFlags::RSV2);
			}
			if reserved[2] {
				flags.insert(dfh::DataFrameFlags::RSV3);
			}
		}

		let masking_key = if mask { Some(mask::gen_mask()) } else { None };

		let header = dfh::DataFrameHeader {
			flags,
			opcode: self.opcode() as u8,
			mask: masking_key,
			len: self.size() as u64,
		};

		let mut data = Vec::<u8>::new();
		dfh::write_header(&mut data, header)?;

		match masking_key {
			Some(mask) => {
				let mut masker = Masker::new(mask, &mut data);
				self.write_payload(&mut masker)?
			}
			None => self.write_payload(&mut data)?,
		};
		writer.write_all(data.as_slice())?;
		Ok(())
	}
}
