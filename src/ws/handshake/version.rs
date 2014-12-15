#[phase(plugin)]
extern crate regex_macros;
extern crate regex;

use std::fmt::{Show, Formatter, Result};
use std::option::Option;

/// Represents an HTTP version.
#[deriving(Clone)]
pub struct HttpVersion {
	/// The major HTTP version.
	pub version_major: u8,
	/// The minor HTTP version if present.
	pub version_minor: Option<u8>,
}

impl HttpVersion {
	/// Create a new HttpVersion from major and minor version numbers.
	pub fn new(version_major: u8, version_minor: Option<u8>) -> HttpVersion {
		HttpVersion {
			version_major: version_major,
			version_minor: version_minor,
		}
	}
	
	/// Create a new HttpVersion from a string. Panics if unable to parse the string.
	pub fn parse(version: &str) -> HttpVersion {
		let re = regex!(r"(\d+)(?:\.(\d+))?");
		let captures = re.captures(version).unwrap();
		
		let version_major: Option<u8> = from_str(captures.at(1));
		let version_minor: Option<u8> = from_str(captures.at(2));
		
		HttpVersion {
			version_major: version_major.unwrap(),
			version_minor: version_minor,
		}
	}
}

impl Show for HttpVersion {
    fn fmt(&self, f: &mut Formatter) -> Result {
		match self.version_minor {
			Some(version_minor) => { write!(f, "{}.{}", self.version_major, version_minor) }
			None => { write!(f, "{}", self.version_major) }
		}
    }
}