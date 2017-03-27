extern crate hyper;
extern crate openssl;

use std::net::TcpStream;
use std::any::Any;
use std::convert::From;
use std::error::Error;
use openssl::ssl::SslStream;
use hyper::http::h1::parse_request;
use hyper::net::{
	NetworkStream,
	HttpStream,
	HttpsStream,
};
use header::{
	WebSocketKey,
	WebSocketVersion,
};
use std::fmt::{
	Formatter,
	Display,
	self,
};
use stream::{
	Stream,
	AsTcpStream,
};
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
	Headers,
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
	UnknownNetworkStream,
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
			&UnknownNetworkStream => "Cannot downcast to known impl of NetworkStream",
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

// TODO: Move this into the main upgrade module
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

		match validate(&request.subject.0, &request.version, &request.headers) {
			Ok(_) => Ok(WsUpgrade {
				stream: self,
				request: request,
			}),
			Err(e) => Err((self, Some(request), e)),
		}
	}
}

// TODO: Move this into the main upgrade module
impl<S> IntoWs for RequestStreamPair<S>
where S: Stream,
{
	type Stream = S;
	type Error = (S, Request, HyperIntoWsError);

	fn into_ws(self) -> Result<WsUpgrade<Self::Stream>, Self::Error> {
		match validate(&self.1.subject.0, &self.1.version, &self.1.headers) {
			Ok(_) => Ok(WsUpgrade {
				stream: self.0,
				request: self.1,
			}),
			Err(e) => Err((self.0, self.1, e)),
		}
	}
}

// TODO
// impl<'a, 'b> IntoWs for HyperRequest<'a, 'b> {
// 	type Stream = Box<AsTcpStream>;
// 	type Error = (HyperRequest<'a, 'b>, HyperIntoWsError);

// 	fn into_ws(self) -> Result<WsUpgrade<Self::Stream>, Self::Error> {
// 		if let Err(e) = validate(&self.method, &self.version, &self.headers) {
// 			return Err((self, e));
// 		}

// 		let stream: Option<Box<AsTcpStream>> = unimplemented!();

// 		if let Some(s) = stream {
// 			Ok(WsUpgrade {
// 				stream: s,
// 				request: Incoming {
// 					version: self.version,
// 					headers: self.headers,
// 					subject: (self.method, self.uri),
// 				},
// 			})
// 		} else {
// 			Err((self, HyperIntoWsError::UnknownNetworkStream))
// 		}
// 	}
// }

pub fn validate(
	method: &Method,
	version: &HttpVersion,
	headers: &Headers
) -> Result<(), HyperIntoWsError>
{
	if *method != Method::Get {
		return Err(HyperIntoWsError::MethodNotGet);
	}

	if *version == HttpVersion::Http09 || *version == HttpVersion::Http10 {
		return Err(HyperIntoWsError::UnsupportedHttpVersion);
	}

	if let Some(version) = headers.get::<WebSocketVersion>() {
		if version != &WebSocketVersion::WebSocket13 {
			return Err(HyperIntoWsError::UnsupportedWebsocketVersion);
		}
	}

	if headers.get::<WebSocketKey>().is_none() {
		return Err(HyperIntoWsError::NoSecWsKeyHeader);
	}

	match headers.get() {
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

	match headers.get() {
		Some(&Connection(ref connection)) => {
			if !check_connection_header(connection) {
				return Err(HyperIntoWsError::NoWsConnectionHeader);
			}
		},
		None => return Err(HyperIntoWsError::NoConnectionHeader),
	};

	Ok(())
}

