//! Describes the generic DataFrame, defining a trait
//! that all dataframes should share. This is so one can
//! optomize the memory footprint of a dataframe for their
//! own needs, and be able to use custom dataframes quickly
use std::io::Write;
use result::WebSocketResult;
use ws::util::header as dfh;
use ws::util::mask::Masker;
use ws::util::mask;

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
	fn write_payload(&self, socket: &mut Write) -> WebSocketResult<()>;

	/// Takes the payload out into a vec
	fn take_payload(self) -> Vec<u8>;

	/// Writes a DataFrame to a Writer.
	fn write_to(&self, writer: &mut Write, mask: bool) -> WebSocketResult<()> {
		let mut flags = dfh::DataFrameFlags::empty();
		if self.is_last() {
			flags.insert(dfh::FIN);
		}
		{
			let reserved = self.reserved();
			if reserved[0] {
				flags.insert(dfh::RSV1);
			}
			if reserved[1] {
				flags.insert(dfh::RSV2);
			}
			if reserved[2] {
				flags.insert(dfh::RSV3);
			}
		}

		let masking_key = if mask { Some(mask::gen_mask()) } else { None };

		let header = dfh::DataFrameHeader {
			flags: flags,
			opcode: self.opcode() as u8,
			mask: masking_key,
			len: self.size() as u64,
		};

		dfh::write_header(writer, header)?;

		match masking_key {
			Some(mask) => {
				let mut masker = Masker::new(mask, writer);
				self.write_payload(&mut masker)?
			}
			None => self.write_payload(writer)?,
		};
		writer.flush()?;
		Ok(())
	}
}
