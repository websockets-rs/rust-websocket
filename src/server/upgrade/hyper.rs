extern crate hyper;

use std::convert::From;
use std::error::Error;
use hyper::http::h1::parse_request;
use hyper::net::NetworkStream;
use header::{
	WebSocketKey,
	WebSocketVersion,
};
use std::fmt::{
	Formatter,
	Display,
	self,
};
use stream::Stream;
use super::{
	IntoWs,
	WsUpgrade,
};
use std::io::{
	Read,
	Write,
	self,
};

pub use hyper::http::h1::Incoming;
pub use hyper::method::Method;
pub use hyper::version::HttpVersion;
pub use hyper::uri::RequestUri;
pub use hyper::buffer::BufReader;
pub use hyper::server::Request as HyperRequest;
pub use hyper::header::{
	Upgrade,
	ProtocolName,
	Connection,
	ConnectionOption,
};

pub type Request = Incoming<(Method, RequestUri)>;

pub struct RequestStreamPair<S: Stream>(pub S, pub Request);

#[derive(Debug)]
pub enum HyperIntoWsError {
	MethodNotGet,
	UnsupportedHttpVersion,
	UnsupportedWebsocketVersion,
	NoSecWsKeyHeader,
	NoWsUpgradeHeader,
	NoUpgradeHeader,
	NoWsConnectionHeader,
	NoConnectionHeader,
	/// IO error from reading the underlying socket
	Io(io::Error),
	/// Error while parsing an incoming request
	Parsing(hyper::error::Error),
}

impl Display for HyperIntoWsError {
	fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
		fmt.write_str(self.description())
	}
}

impl Error for HyperIntoWsError {
	fn description(&self) -> &str {
		use self::HyperIntoWsError::*;
		match self {
			&MethodNotGet => "Request method must be GET",
			&UnsupportedHttpVersion => "Unsupported request HTTP version",
			&UnsupportedWebsocketVersion => "Unsupported WebSocket version",
			&NoSecWsKeyHeader => "Missing Sec-WebSocket-Key header",
			&NoWsUpgradeHeader => "Invalid Upgrade WebSocket header",
			&NoUpgradeHeader => "Missing Upgrade WebSocket header",
			&NoWsConnectionHeader => "Invalid Connection WebSocket header",
			&NoConnectionHeader => "Missing Connection WebSocket header",
			&Io(ref e) => e.description(),
			&Parsing(ref e) => e.description(),
		}
	}

	fn cause(&self) -> Option<&Error> {
		match *self {
			HyperIntoWsError::Io(ref e) => Some(e),
			HyperIntoWsError::Parsing(ref e) => Some(e),
			_ => None,
		}
	}
}

impl From<io::Error> for HyperIntoWsError {
	fn from(err: io::Error) -> Self {
		HyperIntoWsError::Io(err)
	}
}

impl From<hyper::error::Error> for HyperIntoWsError {
	fn from(err: hyper::error::Error) -> Self {
		HyperIntoWsError::Parsing(err)
	}
}

impl<S> IntoWs for S
where S: Stream,
{
	type Stream = S;
	type Error = (Self, Option<Request>, HyperIntoWsError);

	fn into_ws(mut self) -> Result<WsUpgrade<Self::Stream>, Self::Error> {
		let request = {
			let mut reader = BufReader::new(self.reader());
			parse_request(&mut reader)
		};

		let request = match request {
			Ok(r) => r,
			Err(e) => return Err((self, None, e.into())),
		};

		match validate(&request) {
			Ok(_) => Ok(WsUpgrade {
				stream: self,
				request: request,
			}),
			Err(e) => Err((self, Some(request), e)),
		}
	}
}

impl<S> IntoWs for RequestStreamPair<S>
where S: Stream,
{
	type Stream = S;
	type Error = (S, Request, HyperIntoWsError);

	fn into_ws(self) -> Result<WsUpgrade<Self::Stream>, Self::Error> {
		match validate(&self.1) {
			Ok(_) => Ok(WsUpgrade {
				stream: self.0,
				request: self.1,
			}),
			Err(e) => Err((self.0, self.1, e)),
		}
	}
}

// impl<'a, 'b> IntoWs for HyperRequest<'a, 'b> {
// 	type Stream = Box<NetworkStream>;
// 	type Error = (HyperRequest<'a, 'b>, HyperIntoWsError);

// 	fn into_ws(self) -> Result<WsUpgrade<Self::Stream>, Self::Error> {
// 		unimplemented!();
// 	}
// }

pub fn validate(request: &Request) -> Result<(), HyperIntoWsError> {
	if request.subject.0 != Method::Get {
		return Err(HyperIntoWsError::MethodNotGet);
	}

	if request.version == HttpVersion::Http09
		|| request.version == HttpVersion::Http10
	{
		return Err(HyperIntoWsError::UnsupportedHttpVersion);
	}

	if let Some(version) = request.headers.get::<WebSocketVersion>() {
		if version != &WebSocketVersion::WebSocket13 {
			return Err(HyperIntoWsError::UnsupportedWebsocketVersion);
		}
	}

	if request.headers.get::<WebSocketKey>().is_none() {
		return Err(HyperIntoWsError::NoSecWsKeyHeader);
	}

	match request.headers.get() {
		Some(&Upgrade(ref upgrade)) => {
			if upgrade.iter().all(|u| u.name != ProtocolName::WebSocket) {
				return Err(HyperIntoWsError::NoWsUpgradeHeader)
			}
		},
		None => return Err(HyperIntoWsError::NoUpgradeHeader),
	};

	fn check_connection_header(headers: &Vec<ConnectionOption>) -> bool {
		for header in headers {
			if let &ConnectionOption::ConnectionHeader(ref h) = header {
				if h as &str == "upgrade" {
					return true;
				}
			}
		}
		false
	}

	match request.headers.get() {
		Some(&Connection(ref connection)) => {
			if !check_connection_header(connection) {
				return Err(HyperIntoWsError::NoWsConnectionHeader);
			}
		},
		None => return Err(HyperIntoWsError::NoConnectionHeader),
	};

	Ok(())
}
