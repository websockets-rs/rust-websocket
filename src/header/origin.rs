use hyper::header::{Header, HeaderFormat};
use hyper::header::common::util::from_one_raw_str;
use std::fmt::{mod, Show};

/// Represents an Origin header
#[deriving(PartialEq, Clone, Show)]
pub struct Origin(pub String);

impl Deref<String> for Origin {
    fn deref<'a>(&'a self) -> &'a String {
        &self.0
    }
}

impl Header for Origin {
	fn header_name(_: Option<Origin>) -> &'static str {
		"Origin"
	}

	fn parse_header(raw: &[Vec<u8>]) -> Option<Origin> {
		from_one_raw_str(raw).map(|s| Origin(s))
	}
}

impl HeaderFormat for Origin {
	fn fmt_header(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		let Origin(ref value) = *self;
        value.fmt(fmt)
	}
}
