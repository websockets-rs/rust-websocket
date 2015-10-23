use std::io::{Read, Write};
use std::io::Result as IoResult;
use dataframe::Opcode;
use result::{WebSocketResult, WebSocketError};
use ws::util::header as dfh;
use ws::util::mask;

// TODO: Maybe make this a WritableDataFrame
// and make another ReadableDataFrame<D> which
// reads a buffer into D. This is good because it
// also avoids more things called `DataFrame`
// TODO: Remove references to DataFrameTrait
pub trait DataFrame {
    fn is_last(&self) -> bool;
    fn opcode(&self) -> Opcode;
    fn reserved<'a>(&'a self) -> &'a [bool; 3];
    fn payload<'a>(&'a self) -> &'a [u8];

    fn size(&self) -> usize {
        self.payload().len()
    }

    fn write_payload<W>(&self, socket: &mut W) -> WebSocketResult<()>
    where W: Write {
        try!(socket.write_all(self.payload()));
        Ok(())
    }

    /// Writes a DataFrame to a Writer.
    fn write_to<W>(&self, writer: &mut W, mask: bool) -> WebSocketResult<()>
	where W: Write {
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

    	let masking_key = if mask {
            Some(mask::gen_mask())
        } else {
            None
        };

    	let header = dfh::DataFrameHeader {
    		flags: flags,
    		opcode: self.opcode() as u8,
    		mask: masking_key,
    		len: self.size() as u64,
    	};

    	try!(dfh::write_header(writer, header));

    	match masking_key {
            // TODO Optomize this bit
    		Some(mask) => try!(writer.write_all(&mask::mask_data(mask, self.payload())[..])),
    		None => try!(self.write_payload(writer)),
    	};
    	try!(writer.flush());
        Ok(())
    }
}
