use std::fmt::{Show, Formatter, Result};
use std::option::Option;

pub struct HttpVersion {
	pub version_major: u8,
	pub version_minor: Option<u8>,
}

impl HttpVersion {
	pub fn new(version_major: u8, version_minor: Option<u8>) -> HttpVersion {
		HttpVersion {
			version_major: version_major,
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