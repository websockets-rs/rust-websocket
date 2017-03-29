//! Contains the WebSocket client.
extern crate url;

use std::borrow::{
    Cow,
};
use std::net::TcpStream;
use std::net::SocketAddr;
use std::io::Result as IoResult;

use self::url::{
    Url,
    ParseError,
};

use ws;
use ws::sender::Sender as SenderTrait;
use ws::receiver::{
    DataFrameIterator,
    MessageIterator,
};
use ws::receiver::Receiver as ReceiverTrait;
use result::{
    WebSocketResult,
};
use stream::{
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

pub mod builder;
pub use self::builder::ClientBuilder;

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
    pub stream: S,
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

impl<S> Client<S>
    where S: AsTcpStream + Stream,
{
    /// Shuts down the client connection, will cause all pending and future IO to
    /// return immediately with an appropriate value.
    pub fn shutdown(&self) -> IoResult<()> {
        self.stream.as_tcp().shutdown(Shutdown::Both)
    }

    /// See `TcpStream.peer_addr()`.
    pub fn peer_addr(&self) -> IoResult<SocketAddr> {
        self.stream.as_tcp().peer_addr()
	  }

	  /// See `TcpStream.local_addr()`.
	  pub fn local_addr(&self) -> IoResult<SocketAddr> {
        self.stream.as_tcp().local_addr()
	  }

	  /// See `TcpStream.set_nodelay()`.
	  pub fn set_nodelay(&mut self, nodelay: bool) -> IoResult<()> {
        self.stream.as_tcp().set_nodelay(nodelay)
	  }

    /// Changes whether the stream is in nonblocking mode.
    pub fn set_nonblocking(&self, nonblocking: bool) -> IoResult<()> {
        self.stream.as_tcp().set_nonblocking(nonblocking)
    }
}

impl<'u, S> Client<S>
    where S: Stream,
{
    pub fn from_url(address: &'u Url) -> ClientBuilder<'u> {
        ClientBuilder::new(Cow::Borrowed(address))
    }

    pub fn build(address: &str) -> Result<ClientBuilder<'u>, ParseError> {
        let url = try!(Url::parse(address));
        Ok(ClientBuilder::new(Cow::Owned(url)))
    }

    /// Creates a Client from a given stream
    /// **without sending any handshake** this is meant to only be used with
    /// a stream that has a websocket connection already set up.
    /// If in doubt, don't use this!
    pub fn unchecked(stream: S) -> Self {
        Client {
            stream: stream,
            // NOTE: these are always true & false, see
            // https://tools.ietf.org/html/rfc6455#section-5
            sender: Sender::new(true),
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
            stream: read,
            receiver: self.receiver,
        }, Writer {
            stream: write,
            sender: self.sender,
        }))
    }
}
