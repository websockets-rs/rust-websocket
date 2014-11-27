use super::ReadUntilStr;
use std::collections::HashMap;
use std::collections::hash_map::Entries;
use std::ascii::AsciiExt;
use std::iter::Iterator;
use std::option::Option;
use std::string::ToString;
use std::io::IoResult;
use std::clone::Clone;

/// Represents a collection of HTTP headers
pub struct HeaderCollection {
	map: HashMap<String, String>,
}

/// Iterates over the headers present in a HeaderCollection
pub struct Headers<'a> {
	inc: Entries<'a, String, String>,
}

impl HeaderCollection {
	/// Creates a new HeaderCollection
	pub fn new() -> HeaderCollection {
		HeaderCollection {
			map: HashMap::new(),
		}
	}
	
	/// Add the given field-value pair to the collection. If the field is already present, 
	/// the value is appended to the header using comma-separation.
	pub fn insert<A: ToString, B: ToString>(&mut self, field: A, value: B) {
		let string = field.to_string();
		let lowercase = string.as_slice().to_ascii_lower();
		match self.map.insert(lowercase.to_string(), value.to_string()) {
			Some(old_value) => {
				//Append if there's already a value
				self.map.insert(lowercase.to_string(), old_value + ", ".to_string() + value.to_string());
			}
			None => { }
		}
	}
	
	/// Returns true when the specified case-insensitive field name exists in the HeaderCollection.
	pub fn contains_field<A: ToString>(&self, field: A) -> bool {
		let string = field.to_string();
		let lowercase = string.as_slice().to_ascii_lower();
		self.map.contains_key(&(lowercase.to_string()))
	}
	
	/// Gets the value of the header with the specified field name.
	pub fn get<A: ToString>(&self, field: A) -> Option<String> {
		let string = field.to_string();
		let lowercase = string.as_slice().to_ascii_lower();
		match self.map.get(&(lowercase.to_string())) {
			Some(value) => {
				Some(value.to_string())
			}
			None => { None }
		}
	}
	
	/// Removes the header with the specified field name from the HeaderCollection.
	pub fn remove<A: ToString>(&mut self, field: A) -> Option<String> {
		let string = field.to_string();
		let lowercase = string.as_slice().to_ascii_lower();
		match self.map.remove(&(lowercase.to_string())) {
			Some(value) => {
				Some(value.to_string())
			}
			None => { None }
		}
	}
	
	/// Returns an iterator which iterates over each header in the HeaderCollection.
	pub fn iter(&self) -> Headers {
		Headers {
			inc: self.map.iter(),
		}
	}
}

impl Clone for HeaderCollection {
	fn clone(&self) -> HeaderCollection {
		HeaderCollection {
			map: self.map.clone(),
		}
	}
}

impl<'a> Iterator<(String, String)> for Headers<'a> {
	fn next(&mut self) -> Option<(String, String)> {
		match self.inc.next() {
			Some((key, value)) => {
				Some((key.to_string(), value.to_string()))
			}
			None => { None }
		}
	}
}

pub trait ReadHttpHeaders {
	fn read_http_headers(&mut self) -> IoResult<HeaderCollection>;
}

impl<R: Reader> ReadHttpHeaders for R {
	fn read_http_headers(&mut self) -> IoResult<HeaderCollection> {
		let headers_section = try!(self.read_until_str("\r\n\r\n", false));
		let mut headers = HeaderCollection::new();
		for header in headers_section.as_slice().split_str("\r\n") {
			let re = regex!(r"([^:]+)\s*:\s*(.*)");
			let captures = re.captures(header).unwrap();
			
			let property = captures.at(1);
			let value = captures.at(2);
			
			headers.insert(property, value);
		}
		Ok(headers)
	}
}

pub trait WriteHttpHeaders {
	fn write_http_headers(&mut self, headers: &HeaderCollection) -> IoResult<()>;
}

impl<W: Writer> WriteHttpHeaders for W {
	fn write_http_headers(&mut self, headers: &HeaderCollection) -> IoResult<()> {
		for (field, value) in headers.iter() {
			try!(self.write_str(field.as_slice()));
			try!(self.write_str(": "));
			try!(self.write_str(value.as_slice()));
			try!(self.write_str("\r\n"));
		}
		Ok(())
	}
}