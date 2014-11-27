use std::str::from_utf8;
use std::io::{Reader, IoResult, IoError, IoErrorKind};

pub trait ReadUntilStr {
	fn read_until_str(&mut self, until: &str, preserve: bool) -> IoResult<String>;
}

impl<R: Reader> ReadUntilStr for R {
	fn read_until_str(&mut self, until: &str, preserve: bool) -> IoResult<String> {
		let mut input: Vec<u8> = Vec::new();
		let stop = until.as_bytes();
		loop {
			let byte = try!(self.read_byte());
			input.push(byte);
			if input.len() >= stop.len() && input.slice_from(input.len() - until.len()) == stop {
				if !preserve {
					for _ in range(0, until.len()) {
						input.pop();
					}
				}
				break;
			}
		}
		let result = from_utf8(input.as_slice());
		match result {
			Some(result) => { Ok(result.to_string()) }
			None => {
				Err(IoError {
					kind: IoErrorKind::InvalidInput,
					desc: "Invalid UTF-8 sequence",
					detail: None,
				})
			}
		}
	}
}