use std::io::Write;
use std::io::Result as IoResult;
use dataframe::Opcode;

pub trait DataFrame {
    fn is_last(&self) -> bool;
    fn opcode(&self) -> Opcode;
    fn reserved<'a>(&'a self) -> &'a [bool; 3];
    fn write_payload<W>(&self, socket: W) -> IoResult<()>
    where W: Write;
}
