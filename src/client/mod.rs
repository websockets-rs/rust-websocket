//! Contains the WebSocket client.
extern crate url;

use std::borrow::{
    Borrow,
    Cow,
};
use std::net::TcpStream;
use std::marker::PhantomData;
use std::io::Result as IoResult;
use std::io::{
    Read,
    Write,
};
use std::ops::Deref;

use self::url::{
    Url,
    ParseError,
    Position,
};
use openssl::ssl::{
    SslContext,
    SslMethod,
    SslStream,
};
use openssl::ssl::error::SslError;
use hyper::buffer::BufReader;
use hyper::status::StatusCode;
use hyper::http::h1::parse_response;
use hyper::version::HttpVersion;
use hyper::header::{
    Headers,
    Host,
    Connection,
    ConnectionOption,
    Upgrade,
    Protocol,
    ProtocolName,
};
use unicase::UniCase;

use ws;
use ws::sender::Sender as SenderTrait;
use ws::receiver::{
    DataFrameIterator,
    MessageIterator,
};
use ws::receiver::Receiver as ReceiverTrait;
use header::extensions::Extension;
use header::{
    WebSocketAccept,
    WebSocketKey,
    WebSocketVersion,
    WebSocketProtocol,
    WebSocketExtensions,
    Origin
};
use result::{
    WSUrlErrorKind,
    WebSocketResult,
    WebSocketError,
};
use stream::{
    BoxedNetworkStream,
    AsTcpStream,
    Stream,
    Splittable,
    Shutdown,
};
use dataframe::DataFrame;

use ws::dataframe::DataFrame as DataFrameable;
use sender::Sender;
use receiver::Receiver;
pub use sender::Writer;
pub use receiver::Reader;

/// Build clients with a builder-style API
#[derive(Clone, Debug)]
pub struct ClientBuilder<'u, 's> {
    url: Cow<'u, Url>,
    version: HttpVersion,
    headers: Headers,
    version_set: bool,
    key_set: bool,
    ssl_context: Option<Cow<'s, SslContext>>,
}

macro_rules! upsert_header {
    ($headers:expr; $header:ty; {
        Some($pat:pat) => $some_match:expr,
        None => $default:expr
    }) => {{
        match $headers.has::<$header>() {
            true => {
                match $headers.get_mut::<$header>() {
                    Some($pat) => { $some_match; },
                    None => (),
                };
            }
            false => {
                $headers.set($default);
            },
        };
    }}
}

impl<'u, 's> ClientBuilder<'u, 's> {
    pub fn new(url: Cow<'u, Url>) -> Self {
        ClientBuilder {
            url: url,
            version: HttpVersion::Http11,
            version_set: false,
            key_set: false,
            ssl_context: None,
            headers: Headers::new(),
        }
    }

    pub fn add_protocol<P>(mut self, protocol: P) -> Self
        where P: Into<String>,
    {
        upsert_header!(self.headers; WebSocketProtocol; {
            Some(protos) => protos.0.push(protocol.into()),
            None => WebSocketProtocol(vec![protocol.into()])
        });
        self
    }

    pub fn add_protocols<I, S>(mut self, protocols: I) -> Self
        where I: IntoIterator<Item = S>,
              S: Into<String>,
    {
        let mut protocols: Vec<String> = protocols.into_iter()
            .map(Into::into).collect();

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

    pub fn add_extension(mut self, extension: Extension) -> Self
    {
        upsert_header!(self.headers; WebSocketExtensions; {
            Some(protos) => protos.0.push(extension),
            None => WebSocketExtensions(vec![extension])
        });
        self
    }

    pub fn add_extensions<I>(mut self, extensions: I) -> Self
        where I: IntoIterator<Item = Extension>,
    {
        let mut extensions: Vec<Extension> = extensions.into_iter().collect();
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
        where F: Fn(&mut Headers),
    {
        edit(&mut self.headers);
        self
    }

    pub fn ssl_context(mut self, context: &'s SslContext) -> Self {
        self.ssl_context = Some(Cow::Borrowed(context));
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

    fn wrap_ssl(&self, tcp_stream: TcpStream) -> Result<SslStream<TcpStream>, SslError> {
        let context = match self.ssl_context {
            Some(ref ctx) => Cow::Borrowed(ctx.as_ref()),
            None => Cow::Owned(try!(SslContext::new(SslMethod::Tlsv1))),
        };

        SslStream::connect(&*context, tcp_stream)
    }

    pub fn connect(&mut self) -> WebSocketResult<Client<BoxedNetworkStream>> {
        let tcp_stream = try!(self.establish_tcp(None));

        let boxed_stream = if self.url.scheme() == "wss" {
            BoxedNetworkStream(Box::new(try!(self.wrap_ssl(tcp_stream))))
        } else {
            BoxedNetworkStream(Box::new(tcp_stream))
        };

        self.connect_on(boxed_stream)
    }

    pub fn connect_insecure(&mut self) -> WebSocketResult<Client<TcpStream>> {
        let tcp_stream = try!(self.establish_tcp(Some(false)));

        self.connect_on(tcp_stream)
    }

    pub fn connect_secure(&mut self) -> WebSocketResult<Client<SslStream<TcpStream>>> {
        let tcp_stream = try!(self.establish_tcp(Some(true)));

        let ssl_stream = try!(self.wrap_ssl(tcp_stream));

        self.connect_on(ssl_stream)
    }

    // TODO: refactor and split apart into two parts, for when evented happens
    pub fn connect_on<S>(&mut self, mut stream: S) -> WebSocketResult<Client<S>>
        where S: Stream,
    {
        let resource = self.url[Position::BeforePath..Position::AfterQuery]
            .to_owned();

        // enter host if available (unix sockets don't have hosts)
        if let Some(host) = self.url.host_str() {
            self.headers.set(Host {
                hostname: host.to_string(),
                port: self.url.port(),
            });
        }

        self.headers.set(Connection(vec![
            ConnectionOption::ConnectionHeader(UniCase("Upgrade".to_string()))
        ]));

        self.headers.set(Upgrade(vec![Protocol {
            name: ProtocolName::WebSocket,
            // TODO: actually correct or just works?
            version: None
        }]));

        if !self.version_set {
            self.headers.set(WebSocketVersion::WebSocket13);
        }

        if !self.key_set {
            self.headers.set(WebSocketKey::new());
        }

        // send request
        try!(write!(stream.writer(), "GET {} {}\r\n", resource, self.version));
        try!(write!(stream.writer(), "{}\r\n", self.headers));

        // wait for a response
        // TODO: we should buffer it all, how to set up stream for this?
        let response = try!(parse_response(&mut BufReader::new(stream.reader())));
        let status = StatusCode::from_u16(response.subject.0);

        // validate
        if status != StatusCode::SwitchingProtocols {
            return Err(WebSocketError::ResponseError("Status code must be Switching Protocols"));
        }

        let key = try!(self.headers.get::<WebSocketKey>().ok_or(
            WebSocketError::RequestError("Request Sec-WebSocket-Key was invalid")
        ));

        if response.headers.get() != Some(&(WebSocketAccept::new(key))) {
            return Err(WebSocketError::ResponseError("Sec-WebSocket-Accept is invalid"));
        }

        if response.headers.get() != Some(&(Upgrade(vec![Protocol {
            name: ProtocolName::WebSocket,
            version: None
        }]))) {
            return Err(WebSocketError::ResponseError("Upgrade field must be WebSocket"));
        }

        if self.headers.get() != Some(&(Connection(vec![
            ConnectionOption::ConnectionHeader(UniCase("Upgrade".to_string())),
        ]))) {
            return Err(WebSocketError::ResponseError("Connection field must be 'Upgrade'"));
        }

        Ok(Client::new(stream))
    }
}

/// Represents a WebSocket client, which can send and receive messages/data frames.
///
/// `D` is the data frame type, `S` is the type implementing `Sender<D>` and `R`
/// is the type implementing `Receiver<D>`.
///
/// For most cases, the data frame type will be `dataframe::DataFrame`, the Sender
/// type will be `client::Sender<stream::WebSocketStream>` and the receiver type
/// will be `client::Receiver<stream::WebSocketStream>`.
///
/// A `Client` can be split into a `Sender` and a `Receiver` which can then be moved
/// to different threads, often using a send loop and receiver loop concurrently,
/// as shown in the client example in `examples/client.rs`.
///
///#Connecting to a Server
///
///```no_run
///extern crate websocket;
///# fn main() {
///
///use websocket::{Client, Message};
///use websocket::client::request::Url;
///
///let url = Url::parse("ws://127.0.0.1:1234").unwrap(); // Get the URL
///let request = Client::connect(url).unwrap(); // Connect to the server
///let response = request.send().unwrap(); // Send the request
///response.validate().unwrap(); // Ensure the response is valid
///
///let mut client = response.begin(); // Get a Client
///
///let message = Message::text("Hello, World!");
///client.send_message(&message).unwrap(); // Send message
///# }
///```
pub struct Client<S>
    where S: Stream,
{
    stream: S,
    sender: Sender,
    receiver: Receiver,
}

impl Client<TcpStream> {
    /// Shuts down the sending half of the client connection, will cause all pending
    /// and future IO to return immediately with an appropriate value.
    pub fn shutdown_sender(&self) -> IoResult<()> {
        self.stream.as_tcp().shutdown(Shutdown::Write)
    }

    /// Shuts down the receiving half of the client connection, will cause all pending
    /// and future IO to return immediately with an appropriate value.
    pub fn shutdown_receiver(&self) -> IoResult<()> {
        self.stream.as_tcp().shutdown(Shutdown::Read)
    }
}

// TODO: add net2 set_nonblocking and stuff
impl<S> Client<S>
    where S: AsTcpStream + Stream,
{
    /// Shuts down the client connection, will cause all pending and future IO to
    /// return immediately with an appropriate value.
    pub fn shutdown(&self) -> IoResult<()> {
        self.stream.as_tcp().shutdown(Shutdown::Both)
    }
}

impl<'u, 'p, 'e, 's, S> Client<S>
    where S: Stream,
{
    pub fn from_url(address: &'u Url) -> ClientBuilder<'u, 's> {
        ClientBuilder::new(Cow::Borrowed(address))
    }

    pub fn build(address: &str) -> Result<ClientBuilder<'u, 's>, ParseError> {
        let url = try!(Url::parse(address));
        Ok(ClientBuilder::new(Cow::Owned(url)))
    }

    /// Creates a Client from the given Sender and Receiver.
    ///
    /// Esstiallthe opposite of `Client.split()`.
    fn new(stream: S) -> Self {
        Client {
            stream: stream,
            // TODO: always true?
            sender: Sender::new(true),
            // TODO: always false?
            receiver: Receiver::new(false),
        }
    }

    /// Sends a single data frame to the remote endpoint.
    pub fn send_dataframe<D>(&mut self, dataframe: &D) -> WebSocketResult<()>
        where D: DataFrameable
    {
        self.sender.send_dataframe(self.stream.writer(), dataframe)
    }

    /// Sends a single message to the remote endpoint.
    pub fn send_message<'m, M, D>(&mut self, message: &'m M) -> WebSocketResult<()>
        where M: ws::Message<'m, D>, D: DataFrameable
    {
        self.sender.send_message(self.stream.writer(), message)
    }

    /// Reads a single data frame from the remote endpoint.
    pub fn recv_dataframe(&mut self) -> WebSocketResult<DataFrame> {
        self.receiver.recv_dataframe(self.stream.reader())
    }

    /// Returns an iterator over incoming data frames.
    pub fn incoming_dataframes<'a>(&'a mut self) -> DataFrameIterator<'a, Receiver, S::Reader> {
        self.receiver.incoming_dataframes(self.stream.reader())
    }

    /// Reads a single message from this receiver.
    pub fn recv_message<'m, M, I>(&mut self) -> WebSocketResult<M>
        where M: ws::Message<'m, DataFrame, DataFrameIterator = I>, I: Iterator<Item = DataFrame> {
        self.receiver.recv_message(self.stream.reader())
    }

    pub fn stream_ref(&self) -> &S {
        &self.stream
    }

    pub fn stream_ref_mut(&mut self) -> &mut S {
        &mut self.stream
    }

    /// Returns an iterator over incoming messages.
    ///
    ///```no_run
    ///# extern crate websocket;
    ///# fn main() {
    ///use websocket::{Client, Message};
    ///# use websocket::client::request::Url;
    ///# let url = Url::parse("ws://127.0.0.1:1234").unwrap(); // Get the URL
    ///# let request = Client::connect(url).unwrap(); // Connect to the server
    ///# let response = request.send().unwrap(); // Send the request
    ///# response.validate().unwrap(); // Ensure the response is valid
    ///
    ///let mut client = response.begin(); // Get a Client
    ///
    ///for message in client.incoming_messages() {
    ///    let message: Message = message.unwrap();
    ///    println!("Recv: {:?}", message);
    ///}
    ///# }
    ///```
    ///
    /// Note that since this method mutably borrows the `Client`, it may be necessary to
    /// first `split()` the `Client` and call `incoming_messages()` on the returned
    /// `Receiver` to be able to send messages within an iteration.
    ///
    ///```no_run
    ///# extern crate websocket;
    ///# fn main() {
    ///use websocket::{Client, Message, Sender, Receiver};
    ///# use websocket::client::request::Url;
    ///# let url = Url::parse("ws://127.0.0.1:1234").unwrap(); // Get the URL
    ///# let request = Client::connect(url).unwrap(); // Connect to the server
    ///# let response = request.send().unwrap(); // Send the request
    ///# response.validate().unwrap(); // Ensure the response is valid
    ///
    ///let client = response.begin(); // Get a Client
    ///let (mut sender, mut receiver) = client.split(); // Split the Client
    ///for message in receiver.incoming_messages() {
    ///    let message: Message = message.unwrap();
    ///    // Echo the message back
    ///    sender.send_message(&message).unwrap();
    ///}
    ///# }
    ///```
    pub fn incoming_messages<'a, M, D>(&'a mut self) -> MessageIterator<'a, Receiver, D, M, S::Reader>
        where M: ws::Message<'a, D>,
              D: DataFrameable
    {
        self.receiver.incoming_messages(self.stream.reader())
    }
}

impl<S> Client<S>
    where S: Splittable + Stream,
{
    /// Split this client into its constituent Sender and Receiver pair.
    ///
    /// This allows the Sender and Receiver to be sent to different threads.
    ///
    ///```no_run
    ///# extern crate websocket;
    ///# fn main() {
    ///use websocket::{Client, Message, Sender, Receiver};
    ///use std::thread;
    ///# use websocket::client::request::Url;
    ///# let url = Url::parse("ws://127.0.0.1:1234").unwrap(); // Get the URL
    ///# let request = Client::connect(url).unwrap(); // Connect to the server
    ///# let response = request.send().unwrap(); // Send the request
    ///# response.validate().unwrap(); // Ensure the response is valid
    ///
    ///let client = response.begin(); // Get a Client
    ///
    ///let (mut sender, mut receiver) = client.split();
    ///
    ///thread::spawn(move || {
    ///    for message in receiver.incoming_messages() {
    ///        let message: Message = message.unwrap();
    ///        println!("Recv: {:?}", message);
    ///    }
    ///});
    ///
    ///let message = Message::text("Hello, World!");
    ///sender.send_message(&message).unwrap();
    ///# }
    ///```
    pub fn split(self) -> IoResult<(Reader<<S as Splittable>::Reader>, Writer<<S as Splittable>::Writer>)> {
        let (read, write) = try!(self.stream.split());
        Ok((Reader {
            reader: read,
            receiver: self.receiver,
        }, Writer {
            writer: write,
            sender: self.sender,
        }))
    }
}
