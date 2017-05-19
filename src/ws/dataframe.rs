//! Describes the generic DataFrame, defining a trait
//! that all dataframes should share. This is so one can
//! optomize the memory footprint of a dataframe for their
//! own needs, and be able to use custom dataframes quickly
use std::io::Write;
use std::borrow::Cow;
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
	/// Entire payload of the dataframe. If not known then implement
	/// write_payload as that is the actual method used when sending the
	/// dataframe over the wire.
	fn payload(&self) -> Cow<[u8]>;

	/// How long (in bytes) is this dataframe's payload
	fn size(&self) -> usize {
		self.payload().len()
	}

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
	fn write_payload(&self, socket: &mut Write) -> WebSocketResult<()> {
		try!(socket.write_all(&*self.payload()));
		Ok(())
	}

	// TODO: the error can probably be changed to an io error
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

		try!(dfh::write_header(writer, header));

		match masking_key {
			Some(mask) => {
				let mut masker = Masker::new(mask, writer);
				try!(self.write_payload(&mut masker))
			}
			None => try!(self.write_payload(writer)),
		};
		try!(writer.flush());
		Ok(())
	}
}

impl<'a, D> DataFrame for &'a D
    where D: DataFrame
{
	#[inline(always)]
	fn is_last(&self) -> bool {
		D::is_last(self)
	}

	#[inline(always)]
	fn opcode(&self) -> u8 {
		D::opcode(self)
	}

	#[inline(always)]
	fn reserved(&self) -> &[bool; 3] {
		D::reserved(self)
	}

	#[inline(always)]
	fn payload(&self) -> Cow<[u8]> {
		D::payload(self)
	}

	#[inline(always)]
	fn size(&self) -> usize {
		D::size(self)
	}

	#[inline(always)]
	fn write_payload(&self, socket: &mut Write) -> WebSocketResult<()> {
		D::write_payload(self, socket)
	}

	#[inline(always)]
	fn write_to(&self, writer: &mut Write, mask: bool) -> WebSocketResult<()> {
		D::write_to(self, writer, mask)
	}
}
