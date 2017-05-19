//! Everything you need to create a client connection to a websocket.

use std::borrow::Cow;
use std::net::TcpStream;
pub use url::{Url, ParseError};
use url::Position;
use hyper::version::HttpVersion;
use hyper::status::StatusCode;
use hyper::buffer::BufReader;
use hyper::http::h1::parse_response;
use hyper::header::{Headers, Header, HeaderFormat, Host, Connection, ConnectionOption, Upgrade,
                    Protocol, ProtocolName};
use unicase::UniCase;
#[cfg(feature="ssl")]
use openssl::ssl::{SslMethod, SslStream, SslConnector, SslConnectorBuilder};
use header::extensions::Extension;
use header::{WebSocketAccept, WebSocketKey, WebSocketVersion, WebSocketProtocol,
             WebSocketExtensions, Origin};
use result::{WSUrlErrorKind, WebSocketResult, WebSocketError};
#[cfg(feature="ssl")]
use stream::NetworkStream;
use stream::Stream;
use super::Client;

/// Build clients with a builder-style API
/// This makes it easy to create and configure a websocket
/// connection:
///
/// The easiest way to connect is like this:
///
/// ```rust,no_run
/// use websocket::ClientBuilder;
///
/// let client = ClientBuilder::new("ws://myapp.com")
///     .unwrap()
///     .connect_insecure()
///     .unwrap();
/// ```
///
/// But there are so many more possibilities:
///
/// ```rust,no_run
/// use websocket::ClientBuilder;
/// use websocket::header::{Headers, Cookie};
///
/// let default_protos = vec!["ping", "chat"];
/// let mut my_headers = Headers::new();
/// my_headers.set(Cookie(vec!["userid=1".to_owned()]));
///
/// let mut builder = ClientBuilder::new("ws://myapp.com/room/discussion")
///     .unwrap()
///     .add_protocols(default_protos) // any IntoIterator
///     .add_protocol("video-chat")
///     .custom_headers(&my_headers);
///
/// // connect to a chat server with a user
/// let client = builder.connect_insecure().unwrap();
///
/// // clone the builder and take it with you
/// let not_logged_in = builder
///     .clone()
///     .clear_header::<Cookie>()
///     .connect_insecure().unwrap();
/// ```
///
/// You may have noticed we're not using SSL, have no fear, SSL is included!
/// This crate's openssl dependency is optional (and included by default).
/// One can use `connect_secure` to connect to an SSL service, or simply `connect`
/// to choose either SSL or not based on the protocol (`ws://` or `wss://`).
#[derive(Clone, Debug)]
pub struct ClientBuilder<'u> {
	url: Cow<'u, Url>,
	version: HttpVersion,
	headers: Headers,
	version_set: bool,
	key_set: bool,
}

impl<'u> ClientBuilder<'u> {
	/// Create a client builder from an already parsed Url,
	/// because there is no need to parse this will never error.
	///
	/// ```rust
	/// # use websocket::ClientBuilder;
	/// use websocket::url::Url;
	///
	/// // the parsing error will be handled outside the constructor
	/// let url = Url::parse("ws://bitcoins.pizza").unwrap();
	///
	/// let builder = ClientBuilder::from_url(&url);
	/// ```
	/// The path of a URL is optional if no port is given then port
	/// 80 will be used in the case of `ws://` and port `443` will be
	/// used in the case of `wss://`.
	pub fn from_url(address: &'u Url) -> Self {
		ClientBuilder::init(Cow::Borrowed(address))
	}

	/// Create a client builder from a URL string, this will
	/// attempt to parse the URL immediately and return a `ParseError`
	/// if the URL is invalid. URLs must be of the form:
	/// `[ws or wss]://[domain]:[port]/[path]`
	/// The path of a URL is optional if no port is given then port
	/// 80 will be used in the case of `ws://` and port `443` will be
	/// used in the case of `wss://`.
	///
	/// ```rust
	/// # use websocket::ClientBuilder;
	/// let builder = ClientBuilder::new("wss://mycluster.club");
	/// ```
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

	/// Adds a user-defined protocol to the handshake, the server will be
	/// given a list of these protocols and will send back the ones it accepts.
	///
	/// ```rust
	/// # use websocket::ClientBuilder;
	/// # use websocket::header::WebSocketProtocol;
	/// let builder = ClientBuilder::new("wss://my-twitch-clone.rs").unwrap()
	///     .add_protocol("my-chat-proto");
	///
	/// let protos = &builder.get_header::<WebSocketProtocol>().unwrap().0;
	/// assert!(protos.contains(&"my-chat-proto".to_string()));
	/// ```
	pub fn add_protocol<P>(mut self, protocol: P) -> Self
		where P: Into<String>
	{
		upsert_header!(self.headers; WebSocketProtocol; {
            Some(protos) => protos.0.push(protocol.into()),
            None => WebSocketProtocol(vec![protocol.into()])
        });
		self
	}

	/// Adds a user-defined protocols to the handshake.
	/// This can take many kinds of iterators.
	///
	/// ```rust
	/// # use websocket::ClientBuilder;
	/// # use websocket::header::WebSocketProtocol;
	/// let builder = ClientBuilder::new("wss://my-twitch-clone.rs").unwrap()
	///     .add_protocols(vec!["pubsub", "sub.events"]);
	///
	/// let protos = &builder.get_header::<WebSocketProtocol>().unwrap().0;
	/// assert!(protos.contains(&"pubsub".to_string()));
	/// assert!(protos.contains(&"sub.events".to_string()));
	/// ```
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

	/// Removes all the currently set protocols.
	pub fn clear_protocols(mut self) -> Self {
		self.headers.remove::<WebSocketProtocol>();
		self
	}

	/// Adds an extension to the connection.
	/// Unlike protocols, extensions can be below the application level
	/// (like compression). Currently no extensions are supported
	/// out-of-the-box but one can still use them by using their own
	/// implementation. Support is coming soon though.
	///
	/// ```rust
	/// # use websocket::ClientBuilder;
	/// # use websocket::header::{WebSocketExtensions};
	/// # use websocket::header::extensions::Extension;
	/// let builder = ClientBuilder::new("wss://skype-for-linux-lol.com").unwrap()
	///     .add_extension(Extension {
	///         name: "permessage-deflate".to_string(),
	///         params: vec![],
	///     });
	///
	/// let exts = &builder.get_header::<WebSocketExtensions>().unwrap().0;
	/// assert!(exts.first().unwrap().name == "permessage-deflate");
	/// ```
	pub fn add_extension(mut self, extension: Extension) -> Self {
		upsert_header!(self.headers; WebSocketExtensions; {
            Some(protos) => protos.0.push(extension),
            None => WebSocketExtensions(vec![extension])
        });
		self
	}

	/// Adds some extensions to the connection.
	/// Currently no extensions are supported out-of-the-box but one can
	/// still use them by using their own implementation. Support is coming soon though.
	///
	/// ```rust
	/// # use websocket::ClientBuilder;
	/// # use websocket::header::{WebSocketExtensions};
	/// # use websocket::header::extensions::Extension;
	/// let builder = ClientBuilder::new("wss://moxie-chat.org").unwrap()
	///     .add_extensions(vec![
	///         Extension {
	///             name: "permessage-deflate".to_string(),
	///             params: vec![],
	///         },
	///         Extension {
	///             name: "crypt-omemo".to_string(),
	///             params: vec![],
	///         },
	///     ]);
	///
	/// # let exts = &builder.get_header::<WebSocketExtensions>().unwrap().0;
	/// # assert!(exts.first().unwrap().name == "permessage-deflate");
	/// # assert!(exts.last().unwrap().name == "crypt-omemo");
	/// ```
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

	/// Remove all the extensions added to the builder.
	pub fn clear_extensions(mut self) -> Self {
		self.headers.remove::<WebSocketExtensions>();
		self
	}

	/// Add a custom `Sec-WebSocket-Key` header.
	/// Use this only if you know what you're doing, and this almost
	/// never has to be used.
	pub fn key(mut self, key: [u8; 16]) -> Self {
		self.headers.set(WebSocketKey(key));
		self.key_set = true;
		self
	}

	/// Remove the currently set `Sec-WebSocket-Key` header if any.
	pub fn clear_key(mut self) -> Self {
		self.headers.remove::<WebSocketKey>();
		self.key_set = false;
		self
	}

	/// Set the version of the Websocket connection.
	/// Currently this library only supports version 13 (from RFC6455),
	/// but one could use this library to create the handshake then use an
	/// implementation of another websocket version.
	pub fn version(mut self, version: WebSocketVersion) -> Self {
		self.headers.set(version);
		self.version_set = true;
		self
	}

	/// Unset the websocket version to be the default (WebSocket 13).
	pub fn clear_version(mut self) -> Self {
		self.headers.remove::<WebSocketVersion>();
		self.version_set = false;
		self
	}

	/// Sets the Origin header of the handshake.
	/// Normally in browsers this is used to protect against
	/// unauthorized cross-origin use of a WebSocket server, but it is rarely
	/// send by non-browser clients. Still, it can be useful.
	pub fn origin(mut self, origin: String) -> Self {
		self.headers.set(Origin(origin));
		self
	}

	/// Remove the Origin header from the handshake.
	pub fn clear_origin(mut self) -> Self {
		self.headers.remove::<Origin>();
		self
	}

	/// This is a catch all to add random headers to your handshake,
	/// the process here is more manual.
	///
	/// ```rust
	/// # use websocket::ClientBuilder;
	/// # use websocket::header::{Headers, Authorization};
	/// let mut headers = Headers::new();
	/// headers.set(Authorization("let me in".to_owned()));
	///
	/// let builder = ClientBuilder::new("ws://moz.illest").unwrap()
	///     .custom_headers(&headers);
	///
	/// # let hds = &builder.get_header::<Authorization<String>>().unwrap().0;
	/// # assert!(hds == &"let me in".to_string());
	/// ```
	pub fn custom_headers(mut self, custom_headers: &Headers) -> Self {
		self.headers.extend(custom_headers.iter());
		self
	}

	/// Remove a type of header from the handshake, this is to be used
	/// with the catch all `custom_headers`.
	pub fn clear_header<H>(mut self) -> Self
		where H: Header + HeaderFormat
	{
		self.headers.remove::<H>();
		self
	}

	/// Get a header to inspect it.
	pub fn get_header<H>(&self) -> Option<&H>
		where H: Header + HeaderFormat
	{
		self.headers.get::<H>()
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

	/// Connect to a server (finally)!
	/// This will use a `Box<NetworkStream>` to represent either an SSL
	/// connection or a normal TCP connection, what to use will be decided
	/// using the protocol of the URL passed in (e.g. `ws://` or `wss://`)
	///
	/// If you have non-default SSL circumstances, you can use the `ssl_config`
	/// parameter to configure those.
	///
	/// ```rust,no_run
	/// # use websocket::ClientBuilder;
	/// # use websocket::Message;
	/// let mut client = ClientBuilder::new("wss://supersecret.l33t").unwrap()
	///     .connect(None)
	///     .unwrap();
	///
	/// // send messages!
	/// let message = Message::text("m337 47 7pm");
	/// client.send_message(&message).unwrap();
	/// ```
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

	/// Create an insecure (plain TCP) connection to the client.
	/// In this case no `Box` will be used you will just get a TcpStream,
	/// giving you the ability to split the stream into a reader and writer
	/// (since SSL streams cannot be cloned).
	///
	/// ```rust,no_run
	/// # use websocket::ClientBuilder;
	/// let mut client = ClientBuilder::new("wss://supersecret.l33t").unwrap()
	///     .connect_insecure()
	///     .unwrap();
	///
	/// // split into two (for some reason)!
	/// let (receiver, sender) = client.split().unwrap();
	/// ```
	pub fn connect_insecure(&mut self) -> WebSocketResult<Client<TcpStream>> {
		let tcp_stream = try!(self.establish_tcp(Some(false)));

		self.connect_on(tcp_stream)
	}

	/// Create an SSL connection to the sever.
	/// This will only use an `SslStream`, this is useful
	/// when you want to be sure to connect over SSL or when you want access
	/// to the `SslStream` functions (without having to go through a `Box`).
	#[cfg(feature="ssl")]
	pub fn connect_secure(
		&mut self,
		ssl_config: Option<SslConnector>,
	) -> WebSocketResult<Client<SslStream<TcpStream>>> {
		let tcp_stream = try!(self.establish_tcp(Some(true)));

		let ssl_stream = try!(self.wrap_ssl(tcp_stream, ssl_config));

		self.connect_on(ssl_stream)
	}

	// TODO: similar ability for server?
	/// Connects to a websocket server on any stream you would like.
	/// Possible streams:
	///  - Unix Sockets
	///  - Logging Middle-ware
	///  - SSH
	///
	/// ```rust
	/// # use websocket::ClientBuilder;
	/// use websocket::stream::ReadWritePair;
	/// use std::io::Cursor;
	///
	/// let accept = b"HTTP/1.1 101 Switching Protocols\r
	/// Upgrade: websocket\r
	/// Connection: Upgrade\r
	/// Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r
	/// \r\n";
	///
	/// let input = Cursor::new(&accept[..]);
	/// let output = Cursor::new(Vec::new());
	///
	/// let client = ClientBuilder::new("wss://test.ws").unwrap()
	///     .key(b"the sample nonce".clone())
	///     .connect_on(ReadWritePair(input, output))
	///     .unwrap();
	///
	/// let text = (client.into_stream().0).1.into_inner();
	/// let text = String::from_utf8(text).unwrap();
	/// assert!(text.contains("dGhlIHNhbXBsZSBub25jZQ=="), "{}", text);
	/// ```
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
		let mut reader = BufReader::new(stream);
		let response = try!(parse_response(&mut reader));
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

		Ok(Client::unchecked(reader, response.headers, true, false))
	}
}

mod tests {
	#[test]
	fn build_client_with_protocols() {
		use super::*;
		let builder = ClientBuilder::new("ws://127.0.0.1:8080/hello/world")
			.unwrap()
			.add_protocol("protobeard");

		let protos = &builder.headers.get::<WebSocketProtocol>().unwrap().0;
		assert!(protos.contains(&"protobeard".to_string()));
		assert!(protos.len() == 1);

		let builder = ClientBuilder::new("ws://example.org/hello")
			.unwrap()
			.add_protocol("rust-websocket")
			.clear_protocols()
			.add_protocols(vec!["electric", "boogaloo"]);

		let protos = &builder.headers.get::<WebSocketProtocol>().unwrap().0;

		assert!(protos.contains(&"boogaloo".to_string()));
		assert!(protos.contains(&"electric".to_string()));
		assert!(!protos.contains(&"rust-websocket".to_string()));
	}

	// TODO: a few more
}
