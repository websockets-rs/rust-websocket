//! Allows you to take an existing request or stream of data and convert it into a
//! WebSocket client.
extern crate hyper as real_hyper;

use std::error::Error;
use std::net::TcpStream;
use std::io;
use std::io::Result as IoResult;
use std::io::Error as IoError;
use std::io::{
    Write,
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
use header::extensions::Extension;
use header::{
    WebSocketAccept,
	  WebSocketKey,
	  WebSocketVersion,
    WebSocketProtocol,
    WebSocketExtensions,
    Origin,
};
use client::Client;

use unicase::UniCase;
use self::real_hyper::status::StatusCode;
pub use self::real_hyper::http::h1::Incoming;
pub use self::real_hyper::method::Method;
pub use self::real_hyper::version::HttpVersion;
pub use self::real_hyper::uri::RequestUri;
pub use self::real_hyper::buffer::BufReader;
pub use self::real_hyper::http::h1::parse_request;
pub use self::real_hyper::header::{
	  Headers,
	  Upgrade,
    Protocol,
	  ProtocolName,
	  Connection,
	  ConnectionOption,
};

pub mod hyper;

/// Intermediate representation of a half created websocket session.
/// Should be used to examine the client's handshake
/// accept the protocols requested, route the path, etc.
///
/// Users should then call `accept` or `deny` to complete the handshake
/// and start a session.
pub struct WsUpgrade<S>
    where S: Stream,
{
	  stream: S,
	  request: Request,
}

impl<S> WsUpgrade<S>
    where S: Stream,
{
	  pub fn accept(self) -> IoResult<Client<S>> {
        self.accept_with(&Headers::new())
	  }

	  pub fn accept_with(mut self, custom_headers: &Headers) -> IoResult<Client<S>> {
        let mut headers = Headers::new();
        headers.extend(custom_headers.iter());
        headers.set(WebSocketAccept::new(
            // NOTE: we know there is a key because this is a valid request
            // i.e. to construct this you must go through the validate function
            self.request.headers.get::<WebSocketKey>().unwrap()
        ));
		    headers.set(Connection(vec![
			      ConnectionOption::ConnectionHeader(UniCase("Upgrade".to_string()))
		    ]));
        headers.set(Upgrade(vec![
            Protocol::new(ProtocolName::WebSocket, None)
        ]));

        try!(self.send(StatusCode::SwitchingProtocols, &headers));

        Ok(Client::unchecked(self.stream))
	  }

	  pub fn reject(self) -> Result<S, (S, IoError)> {
        self.reject_with(&Headers::new())
	  }

    pub fn reject_with(mut self, headers: &Headers) -> Result<S, (S, IoError)> {
        match self.send(StatusCode::BadRequest, headers) {
            Ok(()) => Ok(self.stream),
            Err(e) => Err((self.stream, e))
        }
    }

    pub fn drop(self) {
        ::std::mem::drop(self);
    }

    pub fn protocols(&self) -> Option<&[String]> {
        self.request.headers.get::<WebSocketProtocol>().map(|p| p.0.as_slice())
    }

    pub fn extensions(&self) -> Option<&[Extension]> {
        self.request.headers.get::<WebSocketExtensions>().map(|e| e.0.as_slice())
    }

    pub fn key(&self) -> Option<&[u8; 16]> {
        self.request.headers.get::<WebSocketKey>().map(|k| &k.0)
    }

    pub fn version(&self) -> Option<&WebSocketVersion> {
        self.request.headers.get::<WebSocketVersion>()
    }

    pub fn origin(&self) -> Option<&str> {
        self.request.headers.get::<Origin>().map(|o| &o.0 as &str)
    }

	  pub fn into_stream(self) -> S {
        self.stream
	  }

    fn send(&mut self,
            status: StatusCode,
            headers: &Headers
    ) -> IoResult<()> {
		    try!(write!(self.stream.writer(), "{} {}\r\n", self.request.version, status));
		    try!(write!(self.stream.writer(), "{}\r\n", headers));
        Ok(())
    }
}

impl<S> WsUpgrade<S>
    where S: Stream + AsTcpStream,
{
	  pub fn tcp_stream(&self) -> &TcpStream {
		    self.stream.as_tcp()
	  }
}

/// Trait to take a stream or similar and attempt to recover the start of a
/// websocket handshake from it.
/// Should be used when a stream might contain a request for a websocket session.
///
/// If an upgrade request can be parsed, one can accept or deny the handshake with
/// the `WsUpgrade` struct.
/// Otherwise the original stream is returned along with an error.
///
/// Note: the stream is owned because the websocket client expects to own its stream.
pub trait IntoWs {
	  type Stream: Stream;
	  type Error;
	  /// Attempt to parse the start of a Websocket handshake, later with the  returned
	  /// `WsUpgrade` struct, call `accept to start a websocket client, and `reject` to
	  /// send a handshake rejection response.
	  fn into_ws(self) -> Result<WsUpgrade<Self::Stream>, Self::Error>;
}


pub type Request = Incoming<(Method, RequestUri)>;
pub struct RequestStreamPair<S: Stream>(pub S, pub Request);

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
	  Parsing(self::real_hyper::error::Error),
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

impl From<real_hyper::error::Error> for HyperIntoWsError {
	  fn from(err: real_hyper::error::Error) -> Self {
		    HyperIntoWsError::Parsing(err)
	  }
}

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
