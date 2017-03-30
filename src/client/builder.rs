use std::borrow::Cow;
use std::net::TcpStream;
pub use url::{Url, ParseError};
use url::Position;
use hyper::version::HttpVersion;
use hyper::status::StatusCode;
use hyper::buffer::BufReader;
use hyper::http::h1::parse_response;
use hyper::header::{Headers, Host, Connection, ConnectionOption, Upgrade, Protocol, ProtocolName};
use unicase::UniCase;
#[cfg(feature="ssl")]
use openssl::ssl::{SslMethod, SslStream, SslConnector, SslConnectorBuilder};
#[cfg(feature="ssl")]
use header::extensions::Extension;
use header::{WebSocketAccept, WebSocketKey, WebSocketVersion, WebSocketProtocol,
             WebSocketExtensions, Origin};
use result::{WSUrlErrorKind, WebSocketResult, WebSocketError};
use stream::{Stream, NetworkStream};
use super::Client;

/// Build clients with a builder-style API
#[derive(Clone, Debug)]
pub struct ClientBuilder<'u> {
	url: Cow<'u, Url>,
	version: HttpVersion,
	headers: Headers,
	version_set: bool,
	key_set: bool,
}

impl<'u> ClientBuilder<'u> {
	pub fn from_url(address: &'u Url) -> Self {
		ClientBuilder::init(Cow::Borrowed(address))
	}

	pub fn new(address: &str) -> Result<Self, ParseError> {
		let url = try!(Url::parse(address));
		Ok(ClientBuilder::init(Cow::Owned(url)))
	}

	fn init(url: Cow<'u, Url>) -> Self {
		ClientBuilder {
			url: url,
			version: HttpVersion::Http11,
			version_set: false,
			key_set: false,
			headers: Headers::new(),
		}
	}

	pub fn add_protocol<P>(mut self, protocol: P) -> Self
		where P: Into<String>
	{
		upsert_header!(self.headers; WebSocketProtocol; {
            Some(protos) => protos.0.push(protocol.into()),
            None => WebSocketProtocol(vec![protocol.into()])
        });
		self
	}

	pub fn add_protocols<I, S>(mut self, protocols: I) -> Self
		where I: IntoIterator<Item = S>,
		      S: Into<String>
	{
		let mut protocols: Vec<String> =
			protocols.into_iter()
			         .map(Into::into)
			         .collect();

		upsert_header!(self.headers; WebSocketProtocol; {
            Some(protos) => protos.0.append(&mut protocols),
            None => WebSocketProtocol(protocols)
        });
		self
	}

	pub fn clear_protocols(mut self) -> Self {
		self.headers.remove::<WebSocketProtocol>();
		self
	}

	pub fn add_extension(mut self, extension: Extension) -> Self {
		upsert_header!(self.headers; WebSocketExtensions; {
            Some(protos) => protos.0.push(extension),
            None => WebSocketExtensions(vec![extension])
        });
		self
	}

	pub fn add_extensions<I>(mut self, extensions: I) -> Self
		where I: IntoIterator<Item = Extension>
	{
		let mut extensions: Vec<Extension> =
			extensions.into_iter().collect();
		upsert_header!(self.headers; WebSocketExtensions; {
            Some(protos) => protos.0.append(&mut extensions),
            None => WebSocketExtensions(extensions)
        });
		self
	}

	pub fn clear_extensions(mut self) -> Self {
		self.headers.remove::<WebSocketExtensions>();
		self
	}

	pub fn key(mut self, key: [u8; 16]) -> Self {
		self.headers.set(WebSocketKey(key));
		self.key_set = true;
		self
	}

	pub fn clear_key(mut self) -> Self {
		self.headers.remove::<WebSocketKey>();
		self.key_set = false;
		self
	}

	pub fn version(mut self, version: WebSocketVersion) -> Self {
		self.headers.set(version);
		self.version_set = true;
		self
	}

	pub fn clear_version(mut self) -> Self {
		self.headers.remove::<WebSocketVersion>();
		self.version_set = false;
		self
	}

	pub fn origin(mut self, origin: String) -> Self {
		self.headers.set(Origin(origin));
		self
	}

	pub fn custom_headers<F>(mut self, edit: F) -> Self
		where F: Fn(&mut Headers)
	{
		edit(&mut self.headers);
		self
	}

	fn establish_tcp(&mut self, secure: Option<bool>) -> WebSocketResult<TcpStream> {
		let port = match (self.url.port(), secure) {
			(Some(port), _) => port,
			(None, None) if self.url.scheme() == "wss" => 443,
			(None, None) => 80,
			(None, Some(true)) => 443,
			(None, Some(false)) => 80,
		};
		let host = match self.url.host_str() {
			Some(h) => h,
			None => return Err(WebSocketError::WebSocketUrlError(WSUrlErrorKind::NoHostName)),
		};

		let tcp_stream = try!(TcpStream::connect((host, port)));
		Ok(tcp_stream)
	}

	#[cfg(feature="ssl")]
	fn wrap_ssl(
		&self,
		tcp_stream: TcpStream,
		connector: Option<SslConnector>,
	) -> WebSocketResult<SslStream<TcpStream>> {
		let host = match self.url.host_str() {
			Some(h) => h,
			None => return Err(WebSocketError::WebSocketUrlError(WSUrlErrorKind::NoHostName)),
		};
		let connector = match connector {
			Some(c) => c,
			None => try!(SslConnectorBuilder::new(SslMethod::tls())).build(),
		};

		let ssl_stream = try!(connector.connect(host, tcp_stream));
		Ok(ssl_stream)
	}

	#[cfg(feature="ssl")]
	pub fn connect(
		&mut self,
		ssl_config: Option<SslConnector>,
	) -> WebSocketResult<Client<Box<NetworkStream>>> {
		let tcp_stream = try!(self.establish_tcp(None));

		let boxed_stream: Box<NetworkStream> = if
			self.url.scheme() == "wss" {
			Box::new(try!(self.wrap_ssl(tcp_stream, ssl_config)))
		} else {
			Box::new(tcp_stream)
		};

		self.connect_on(boxed_stream)
	}

	pub fn connect_insecure(&mut self) -> WebSocketResult<Client<TcpStream>> {
		let tcp_stream = try!(self.establish_tcp(Some(false)));

		self.connect_on(tcp_stream)
	}

	#[cfg(feature="ssl")]
	pub fn connect_secure(
		&mut self,
		ssl_config: Option<SslConnector>,
	) -> WebSocketResult<Client<SslStream<TcpStream>>> {
		let tcp_stream = try!(self.establish_tcp(Some(true)));

		let ssl_stream = try!(self.wrap_ssl(tcp_stream, ssl_config));

		self.connect_on(ssl_stream)
	}

	pub fn connect_on<S>(&mut self, mut stream: S) -> WebSocketResult<Client<S>>
		where S: Stream
	{
		let resource = self.url[Position::BeforePath..Position::AfterQuery].to_owned();

		// enter host if available (unix sockets don't have hosts)
		if let Some(host) = self.url.host_str() {
			self.headers
			    .set(Host {
			             hostname: host.to_string(),
			             port: self.url.port(),
			         });
		}

		self.headers
		    .set(Connection(vec![
            ConnectionOption::ConnectionHeader(UniCase("Upgrade".to_string()))
        ]));

		self.headers
		    .set(Upgrade(vec![
			Protocol {
				name: ProtocolName::WebSocket,
				version: None,
			},
		]));

		if !self.version_set {
			self.headers.set(WebSocketVersion::WebSocket13);
		}

		if !self.key_set {
			self.headers.set(WebSocketKey::new());
		}

		// send request
		try!(write!(stream, "GET {} {}\r\n", resource, self.version));
		try!(write!(stream, "{}\r\n", self.headers));

		// wait for a response
		// TODO: some extra data might get lost with this reader, try to avoid #72
		let response = try!(parse_response(&mut BufReader::new(&mut stream)));
		let status = StatusCode::from_u16(response.subject.0);

		// validate
		if status != StatusCode::SwitchingProtocols {
			return Err(WebSocketError::ResponseError("Status code must be Switching Protocols"));
		}

		let key = try!(self.headers
		                   .get::<WebSocketKey>()
		                   .ok_or(WebSocketError::RequestError("Request Sec-WebSocket-Key was invalid")));

		if response.headers.get() != Some(&(WebSocketAccept::new(key))) {
			return Err(WebSocketError::ResponseError("Sec-WebSocket-Accept is invalid"));
		}

		if response.headers.get() !=
		   Some(&(Upgrade(vec![
			Protocol {
				name: ProtocolName::WebSocket,
				version: None,
			},
		]))) {
			return Err(WebSocketError::ResponseError("Upgrade field must be WebSocket"));
		}

		if self.headers.get() !=
		   Some(&(Connection(vec![
            ConnectionOption::ConnectionHeader(UniCase("Upgrade".to_string())),
        ]))) {
			return Err(WebSocketError::ResponseError("Connection field must be 'Upgrade'"));
		}

		Ok(Client::unchecked(stream, response.headers))
	}
}
