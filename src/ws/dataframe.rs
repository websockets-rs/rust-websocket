use std::io::Write;
use std::io::Result as IoResult;
use dataframe::Opcode;

pub trait DataFrame {
    fn is_last(&self) -> bool;
    fn opcode(&self) -> Opcode;
    fn reserved<'a>(&'a self) -> &'a [bool; 3];
    fn payload<'a>(&'a self) -> &'a [u8];
    fn write_payload<W>(&self, socket: &mut W) -> IoResult<()>
    where W: Write;

    // TODO: Move util write dataframe fn's here as premade methods
}
