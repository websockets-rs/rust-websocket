//! Everything you need to create a client connection to a websocket.

use header::extensions::Extension;
use header::{Origin, WebSocketExtensions, WebSocketKey, WebSocketProtocol, WebSocketVersion};
use hyper::header::{Header, HeaderFormat, Headers};
use hyper::version::HttpVersion;
use std::borrow::Cow;
pub use url::{ParseError, Url};

#[cfg(any(feature = "sync", feature = "async"))]
mod common_imports {
	pub use header::WebSocketAccept;
	pub use hyper::buffer::BufReader;
	pub use hyper::header::{Connection, ConnectionOption, Host, Protocol, ProtocolName, Upgrade};
	pub use hyper::http::h1::parse_response;
	pub use hyper::http::h1::Incoming;
	pub use hyper::http::RawStatus;
	pub use hyper::method::Method;
	pub use hyper::status::StatusCode;
	pub use hyper::uri::RequestUri;
	pub use result::{WSUrlErrorKind, WebSocketError, WebSocketResult};
	pub use std::net::TcpStream;
	pub use std::net::ToSocketAddrs;
	pub use stream::{self, Stream};
	pub use unicase::UniCase;
	pub use url::Position;
}
#[cfg(any(feature = "sync", feature = "async"))]
use self::common_imports::*;

#[cfg(feature = "sync")]
use super::sync::Client;

#[cfg(feature = "sync-ssl")]
use stream::sync::NetworkStream;

#[cfg(any(feature = "sync-ssl", feature = "async-ssl"))]
use native_tls::TlsConnector;
#[cfg(feature = "sync-ssl")]
use native_tls::TlsStream;

#[cfg(feature = "async")]
mod async_imports {
	pub use super::super::async;
	pub use codec::ws::{Context, MessageCodec};
	pub use futures::future;
	pub use futures::Stream as FutureStream;
	pub use futures::{Future, Sink};
	pub use tokio_core::net::TcpStreamNew;
	pub use tokio_core::reactor::Handle;
	pub use tokio_io::codec::Framed;
	#[cfg(feature = "async-ssl")]
	pub use tokio_tls::TlsConnectorExt;
}
#[cfg(feature = "async")]
use self::async_imports::*;

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
		let url = Url::parse(address)?;
		Ok(ClientBuilder::init(Cow::Owned(url)))
	}

	fn init(url: Cow<'u, Url>) -> Self {
		ClientBuilder {
			url,
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
	where
		P: Into<String>,
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
	where
		I: IntoIterator<Item = S>,
		S: Into<String>,
	{
		let mut protocols: Vec<String> = protocols.into_iter().map(Into::into).collect();

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
	where
		I: IntoIterator<Item = Extension>,
	{
		let mut extensions: Vec<Extension> = extensions.into_iter().collect();
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
	where
		H: Header + HeaderFormat,
	{
		self.headers.remove::<H>();
		self
	}

	/// Get a header to inspect it.
	pub fn get_header<H>(&self) -> Option<&H>
	where
		H: Header + HeaderFormat,
	{
		self.headers.get::<H>()
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
	#[cfg(feature = "sync-ssl")]
	pub fn connect(
		&mut self,
		ssl_config: Option<TlsConnector>,
	) -> WebSocketResult<Client<Box<NetworkStream + Send>>> {
		let tcp_stream = self.establish_tcp(None)?;

		let boxed_stream: Box<NetworkStream + Send> = if self.url.scheme() == "wss" {
			Box::new(self.wrap_ssl(tcp_stream, ssl_config)?)
		} else {
			Box::new(tcp_stream)
		};

		self.connect_on(boxed_stream)
	}

	/// Create an insecure (plain TCP) connection to the client.
	/// In this case no `Box` will be used, you will just get a TcpStream,
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
	#[cfg(feature = "sync")]
	pub fn connect_insecure(&mut self) -> WebSocketResult<Client<TcpStream>> {
		let tcp_stream = self.establish_tcp(Some(false))?;

		self.connect_on(tcp_stream)
	}

	/// Create an SSL connection to the sever.
	/// This will only use an `TlsStream`, this is useful
	/// when you want to be sure to connect over SSL or when you want access
	/// to the `TlsStream` functions (without having to go through a `Box`).
	#[cfg(feature = "sync-ssl")]
	pub fn connect_secure(
		&mut self,
		ssl_config: Option<TlsConnector>,
	) -> WebSocketResult<Client<TlsStream<TcpStream>>> {
		let tcp_stream = self.establish_tcp(Some(true))?;

		let ssl_stream = self.wrap_ssl(tcp_stream, ssl_config)?;

		self.connect_on(ssl_stream)
	}

	/// Connects to a websocket server on any stream you would like.
	/// Possible streams:
	///  - Unix Sockets
	///  - Logging Middle-ware
	///  - SSH
	///
	/// ```rust
	/// # use websocket::ClientBuilder;
	/// use websocket::sync::stream::ReadWritePair;
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
	#[cfg(feature = "sync")]
	pub fn connect_on<S>(&mut self, mut stream: S) -> WebSocketResult<Client<S>>
	where
		S: Stream,
	{
		// send request
		let resource = self.build_request();
		let data = format!("GET {} {}\r\n{}\r\n", resource, self.version, self.headers);
		stream.write_all(data.as_bytes())?;

		// wait for a response
		let mut reader = BufReader::new(stream);
		let response = parse_response(&mut reader)?;

		// validate
		self.validate(&response)?;

		Ok(Client::unchecked(reader, response.headers, true, false))
	}

	/// Connect to a websocket server asynchronously.
	///
	/// This will use a `Box<AsyncRead + AsyncWrite + Send>` to represent either
	/// an SSL connection or a normal TCP connection, what to use will be decided
	/// using the protocol of the URL passed in (e.g. `ws://` or `wss://`)
	///
	/// If you have non-default SSL circumstances, you can use the `ssl_config`
	/// parameter to configure those.
	///
	///# Example
	///
	/// ```rust,no_run
	/// # extern crate rand;
	/// # extern crate tokio_core;
	/// # extern crate futures;
	/// # extern crate websocket;
	/// use websocket::ClientBuilder;
	/// use websocket::futures::{Future, Stream, Sink};
	/// use websocket::Message;
	/// use tokio_core::reactor::Core;
	/// # use rand::Rng;
	///
	/// # fn main() {
	/// let mut core = Core::new().unwrap();
	///
	/// // let's randomly do either SSL or plaintext
	/// let url = if rand::thread_rng().gen() {
	///     "ws://echo.websocket.org"
	/// } else {
	///     "wss://echo.websocket.org"
	/// };
	///
	/// // send a message and hear it come back
	/// let echo_future = ClientBuilder::new(url).unwrap()
	///     .async_connect(None, &core.handle())
	///     .and_then(|(s, _)| s.send(Message::text("hallo").into()))
	///     .and_then(|s| s.into_future().map_err(|e| e.0))
	///     .map(|(m, _)| {
	///         assert_eq!(m, Some(Message::text("hallo").into()))
	///     });
	///
	/// core.run(echo_future).unwrap();
	/// # }
	/// ```
	#[cfg(feature = "async-ssl")]
	pub fn async_connect(
		self,
		ssl_config: Option<TlsConnector>,
		handle: &Handle,
	) -> async::ClientNew<Box<stream::async::Stream + Send>> {
		// connect to the tcp stream
		let tcp_stream = match self.async_tcpstream(None, handle) {
			Ok(t) => t,
			Err(e) => return Box::new(future::err(e)),
		};

		let builder = ClientBuilder {
			url: Cow::Owned(self.url.into_owned()),
			version: self.version,
			headers: self.headers,
			version_set: self.version_set,
			key_set: self.key_set,
		};

		// check if we should connect over ssl or not
		if builder.url.scheme() == "wss" {
			// configure the tls connection
			let (host, connector) = {
				match builder.extract_host_ssl_conn(ssl_config) {
					Ok((h, conn)) => (h.to_string(), conn),
					Err(e) => return Box::new(future::err(e)),
				}
			};
			// secure connection, wrap with ssl
			let future = tcp_stream
				.map_err(|e| e.into())
				.and_then(move |s| connector.connect_async(&host, s).map_err(|e| e.into()))
				.and_then(move |stream| {
					let stream: Box<stream::async::Stream + Send> = Box::new(stream);
					builder.async_connect_on(stream)
				});
			Box::new(future)
		} else {
			// insecure connection, connect normally
			let future = tcp_stream.map_err(|e| e.into()).and_then(move |stream| {
				let stream: Box<stream::async::Stream + Send> = Box::new(stream);
				builder.async_connect_on(stream)
			});
			Box::new(future)
		}
	}

	/// Asynchronously create an SSL connection to a websocket sever.
	///
	/// This method will only try to connect over SSL and fail otherwise, useful
	/// when you want to be sure to connect over SSL or when you want access
	/// to the `TlsStream` functions (without having to go through a `Box`).
	///
	/// If you have non-default SSL circumstances, you can use the `ssl_config`
	/// parameter to configure those.
	///
	///# Example
	///
	/// ```rust
	/// # extern crate tokio_core;
	/// # extern crate futures;
	/// # extern crate websocket;
	/// use websocket::ClientBuilder;
	/// use websocket::futures::{Future, Stream, Sink};
	/// use websocket::Message;
	/// use tokio_core::reactor::Core;
	/// # fn main() {
	///
	/// let mut core = Core::new().unwrap();
	///
	/// // send a message and hear it come back
	/// let echo_future = ClientBuilder::new("wss://echo.websocket.org").unwrap()
	///     .async_connect_secure(None, &core.handle())
	///     .and_then(|(s, _)| s.send(Message::text("hallo").into()))
	///     .and_then(|s| s.into_future().map_err(|e| e.0))
	///     .map(|(m, _)| {
	///         assert_eq!(m, Some(Message::text("hallo").into()))
	///     });
	///
	/// core.run(echo_future).unwrap();
	/// # }
	/// ```
	#[cfg(feature = "async-ssl")]
	pub fn async_connect_secure(
		self,
		ssl_config: Option<TlsConnector>,
		handle: &Handle,
	) -> async::ClientNew<async::TlsStream<async::TcpStream>> {
		// connect to the tcp stream
		let tcp_stream = match self.async_tcpstream(Some(true), handle) {
			Ok(t) => t,
			Err(e) => return Box::new(future::err(e)),
		};

		// configure the tls connection
		let (host, connector) = {
			match self.extract_host_ssl_conn(ssl_config) {
				Ok((h, conn)) => (h.to_string(), conn),
				Err(e) => return Box::new(future::err(e)),
			}
		};

		let builder = ClientBuilder {
			url: Cow::Owned(self.url.into_owned()),
			version: self.version,
			headers: self.headers,
			version_set: self.version_set,
			key_set: self.key_set,
		};

		// put it all together
		let future = tcp_stream
			.map_err(|e| e.into())
			.and_then(move |s| connector.connect_async(&host, s).map_err(|e| e.into()))
			.and_then(move |stream| builder.async_connect_on(stream));
		Box::new(future)
	}

	// TODO: add conveniences like .response_to_pings, .send_close, etc.
	/// Asynchronously create an insecure (plain TCP) connection to the client.
	///
	/// In this case no `Box` will be used, you will just get a `TcpStream`,
	/// giving you less allocations on the heap and direct access to `TcpStream`
	/// functions.
	///
	///# Example
	///
	/// ```rust,no_run
	/// # extern crate tokio_core;
	/// # extern crate futures;
	/// # extern crate websocket;
	/// use websocket::ClientBuilder;
	/// use websocket::futures::{Future, Stream, Sink};
	/// use websocket::Message;
	/// use tokio_core::reactor::Core;
	/// # fn main() {
	///
	/// let mut core = Core::new().unwrap();
	///
	/// // send a message and hear it come back
	/// let echo_future = ClientBuilder::new("ws://echo.websocket.org").unwrap()
	///     .async_connect_insecure(&core.handle())
	///     .and_then(|(s, _)| s.send(Message::text("hallo").into()))
	///     .and_then(|s| s.into_future().map_err(|e| e.0))
	///     .map(|(m, _)| {
	///         assert_eq!(m, Some(Message::text("hallo").into()))
	///     });
	///
	/// core.run(echo_future).unwrap();
	/// # }
	/// ```
	#[cfg(feature = "async")]
	pub fn async_connect_insecure(self, handle: &Handle) -> async::ClientNew<async::TcpStream> {
		let tcp_stream = match self.async_tcpstream(Some(false), handle) {
			Ok(t) => t,
			Err(e) => return Box::new(future::err(e)),
		};

		let builder = ClientBuilder {
			url: Cow::Owned(self.url.into_owned()),
			version: self.version,
			headers: self.headers,
			version_set: self.version_set,
			key_set: self.key_set,
		};

		let future = tcp_stream
			.map_err(|e| e.into())
			.and_then(move |stream| builder.async_connect_on(stream));
		Box::new(future)
	}

	/// Asynchronously connects to a websocket server on any stream you would like.
	/// Possible streams:
	///  - Unix Sockets
	///  - Bluetooth
	///  - Logging Middle-ware
	///  - SSH
	///
	/// The stream must be `AsyncRead + AsyncWrite + Send + 'static`.
	///
	/// # Example
	///
	/// ```rust
	/// use websocket::header::WebSocketProtocol;
	/// use websocket::ClientBuilder;
	/// use websocket::sync::stream::ReadWritePair;
	/// use websocket::futures::Future;
	/// use websocket::async::Core;
	/// # use std::io::Cursor;
	///
	/// let mut core = Core::new().unwrap();
	///
	/// let accept = b"\
	/// HTTP/1.1 101 Switching Protocols\r\n\
	/// Upgrade: websocket\r\n\
	/// Sec-WebSocket-Protocol: proto-metheus\r\n\
	/// Connection: Upgrade\r\n\
	/// Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\
	/// \r\n";
	///
	/// let input = Cursor::new(&accept[..]);
	/// let output = Cursor::new(Vec::new());
	///
	/// let client = ClientBuilder::new("wss://test.ws").unwrap()
	///     .key(b"the sample nonce".clone())
	///     .async_connect_on(ReadWritePair(input, output))
	///     .map(|(_, headers)| {
	///         let proto: &WebSocketProtocol = headers.get().unwrap();
	///         assert_eq!(proto.0.first().unwrap(), "proto-metheus")
	///     });
	///
	/// core.run(client).unwrap();
	/// ```
	#[cfg(feature = "async")]
	pub fn async_connect_on<S>(self, stream: S) -> async::ClientNew<S>
	where
		S: stream::async::Stream + Send + 'static,
	{
		let mut builder = ClientBuilder {
			url: Cow::Owned(self.url.into_owned()),
			version: self.version,
			headers: self.headers,
			version_set: self.version_set,
			key_set: self.key_set,
		};
		let resource = builder.build_request();
		let framed = stream.framed(::codec::http::HttpClientCodec);
		let request = Incoming {
			version: builder.version,
			headers: builder.headers.clone(),
			subject: (Method::Get, RequestUri::AbsolutePath(resource)),
		};

		let future = framed
			// send request
			.send(request)
			.map_err(::std::convert::Into::into)
			// wait for a response
			.and_then(|stream| stream.into_future().map_err(|e| e.0.into()))
			// validate
			.and_then(move |(message, stream)| {
				message
					.ok_or(WebSocketError::ProtocolError(
						"Connection closed before handshake could complete.",
					))
					.and_then(|message| builder.validate(&message).map(|()| (message, stream)))
			})
			// output the final client and metadata
			.map(|(message, stream)| {
				let codec = MessageCodec::default(Context::Client);
				let client = Framed::from_parts(stream.into_parts(), codec);
				(client, message.headers)
			});

		Box::new(future)
	}

	#[cfg(feature = "async")]
	fn async_tcpstream(
		&self,
		secure: Option<bool>,
		handle: &Handle,
	) -> WebSocketResult<TcpStreamNew> {
		// get the address to connect to, return an error future if ther's a problem
		let address = match self
			.extract_host_port(secure)
			.and_then(|p| Ok(p.to_socket_addrs()?))
		{
			Ok(mut s) => match s.next() {
				Some(a) => a,
				None => {
					return Err(WebSocketError::WebSocketUrlError(
						WSUrlErrorKind::NoHostName,
					));
				}
			},
			Err(e) => return Err(e),
		};

		// connect a tcp stream
		Ok(async::TcpStream::connect(&address, handle))
	}

	#[cfg(any(feature = "sync", feature = "async"))]
	fn build_request(&mut self) -> String {
		// enter host if available (unix sockets don't have hosts)
		if let Some(host) = self.url.host_str() {
			self.headers.set(Host {
				hostname: host.to_string(),
				port: self.url.port(),
			});
		}

		self.headers
			.set(Connection(vec![ConnectionOption::ConnectionHeader(
				UniCase("Upgrade".to_string()),
			)]));

		self.headers.set(Upgrade(vec![Protocol {
			name: ProtocolName::WebSocket,
			version: None,
		}]));

		if !self.version_set {
			self.headers.set(WebSocketVersion::WebSocket13);
		}

		if !self.key_set {
			self.headers.set(WebSocketKey::new());
		}

		// send request
		self.url[Position::BeforePath..Position::AfterQuery].to_owned()
	}

	#[cfg(any(feature = "sync", feature = "async"))]
	fn validate(&self, response: &Incoming<RawStatus>) -> WebSocketResult<()> {
		let status = StatusCode::from_u16(response.subject.0);

		if status != StatusCode::SwitchingProtocols {
			return Err(WebSocketError::ResponseError(
				"Status code must be Switching Protocols",
			));
		}

		let key = self
			.headers
			.get::<WebSocketKey>()
			.ok_or(WebSocketError::RequestError(
				"Request Sec-WebSocket-Key was invalid",
			))?;

		if response.headers.get() != Some(&(WebSocketAccept::new(key))) {
			return Err(WebSocketError::ResponseError(
				"Sec-WebSocket-Accept is invalid",
			));
		}

		if response.headers.get()
			!= Some(
				&(Upgrade(vec![Protocol {
					name: ProtocolName::WebSocket,
					version: None,
				}])),
			) {
			return Err(WebSocketError::ResponseError(
				"Upgrade field must be WebSocket",
			));
		}

		if self.headers.get()
			!= Some(
				&(Connection(vec![ConnectionOption::ConnectionHeader(UniCase(
					"Upgrade".to_string(),
				))])),
			) {
			return Err(WebSocketError::ResponseError(
				"Connection field must be 'Upgrade'",
			));
		}

		Ok(())
	}

	#[cfg(any(feature = "sync", feature = "async"))]
	fn extract_host_port(&self, secure: Option<bool>) -> WebSocketResult<(&str, u16)> {
		let port = match (self.url.port(), secure) {
			(Some(port), _) => port,
			(None, None) if self.url.scheme() == "wss" => 443,
			(None, None) => 80,
			(None, Some(true)) => 443,
			(None, Some(false)) => 80,
		};
		let host = match self.url.host_str() {
			Some(h) => h,
			None => {
				return Err(WebSocketError::WebSocketUrlError(
					WSUrlErrorKind::NoHostName,
				))
			}
		};

		Ok((host, port))
	}

	#[cfg(feature = "sync")]
	fn establish_tcp(&mut self, secure: Option<bool>) -> WebSocketResult<TcpStream> {
		Ok(TcpStream::connect(self.extract_host_port(secure)?)?)
	}

	#[cfg(any(feature = "sync-ssl", feature = "async-ssl"))]
	fn extract_host_ssl_conn(
		&self,
		connector: Option<TlsConnector>,
	) -> WebSocketResult<(&str, TlsConnector)> {
		let host = match self.url.host_str() {
			Some(h) => h,
			None => {
				return Err(WebSocketError::WebSocketUrlError(
					WSUrlErrorKind::NoHostName,
				))
			}
		};
		let connector = match connector {
			Some(c) => c,
			None => TlsConnector::builder()?.build()?,
		};
		Ok((host, connector))
	}

	#[cfg(feature = "sync-ssl")]
	fn wrap_ssl(
		&self,
		tcp_stream: TcpStream,
		connector: Option<TlsConnector>,
	) -> WebSocketResult<TlsStream<TcpStream>> {
		let (host, connector) = self.extract_host_ssl_conn(connector)?;
		let ssl_stream = connector.connect(host, tcp_stream)?;
		Ok(ssl_stream)
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
}
