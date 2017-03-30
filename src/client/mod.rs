//! Contains the WebSocket client.
extern crate url;

use std::net::TcpStream;
use std::net::SocketAddr;
use std::io::Result as IoResult;
use hyper::header::Headers;

use ws;
use ws::sender::Sender as SenderTrait;
use ws::receiver::{DataFrameIterator, MessageIterator};
use ws::receiver::Receiver as ReceiverTrait;
use result::WebSocketResult;
use stream::{AsTcpStream, Stream, Splittable, Shutdown};
use dataframe::DataFrame;
use header::{WebSocketProtocol, WebSocketExtensions, Origin};
use header::extensions::Extension;

use ws::dataframe::DataFrame as DataFrameable;
use sender::Sender;
use receiver::Receiver;
pub use sender::Writer;
pub use receiver::Reader;

pub mod builder;
pub use self::builder::{ClientBuilder, Url, ParseError};

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
///use websocket::{ClientBuilder, Message};
///
///let mut client = ClientBuilder::new("ws://127.0.0.1:1234").unwrap()
///                     .connect(None).unwrap();
///
///let message = Message::text("Hello, World!");
///client.send_message(&message).unwrap(); // Send message
///# }
///```
pub struct Client<S>
	where S: Stream
{
	pub stream: S,
	headers: Headers,
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
    where S: AsTcpStream + Stream
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

impl<S> Client<S>
    where S: Stream
{
	/// Creates a Client from a given stream
	/// **without sending any handshake** this is meant to only be used with
	/// a stream that has a websocket connection already set up.
	/// If in doubt, don't use this!
	pub fn unchecked(stream: S, headers: Headers) -> Self {
		Client {
			headers: headers,
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
		self.sender.send_dataframe(&mut self.stream, dataframe)
	}

	/// Sends a single message to the remote endpoint.
	pub fn send_message<'m, M, D>(&mut self, message: &'m M) -> WebSocketResult<()>
		where M: ws::Message<'m, D>,
		      D: DataFrameable
	{
		self.sender.send_message(&mut self.stream, message)
	}

	/// Reads a single data frame from the remote endpoint.
	pub fn recv_dataframe(&mut self) -> WebSocketResult<DataFrame> {
		self.receiver.recv_dataframe(&mut self.stream)
	}

	/// Returns an iterator over incoming data frames.
	pub fn incoming_dataframes<'a>(&'a mut self) -> DataFrameIterator<'a, Receiver, S> {
		self.receiver.incoming_dataframes(&mut self.stream)
	}

	/// Reads a single message from this receiver.
	pub fn recv_message<'m, M, I>(&mut self) -> WebSocketResult<M>
		where M: ws::Message<'m, DataFrame, DataFrameIterator = I>,
		      I: Iterator<Item = DataFrame>
	{
		self.receiver.recv_message(&mut self.stream)
	}

	pub fn headers(&self) -> &Headers {
		&self.headers
	}

	pub fn protocols(&self) -> &[String] {
		self.headers
		    .get::<WebSocketProtocol>()
		    .map(|p| p.0.as_slice())
		    .unwrap_or(&[])
	}

	pub fn extensions(&self) -> &[Extension] {
		self.headers
		    .get::<WebSocketExtensions>()
		    .map(|e| e.0.as_slice())
		    .unwrap_or(&[])
	}

	pub fn origin(&self) -> Option<&str> {
		self.headers.get::<Origin>().map(|o| &o.0 as &str)
	}

	pub fn stream_ref(&self) -> &S {
		&self.stream
	}

	pub fn stream_ref_mut(&mut self) -> &mut S {
		&mut self.stream
	}

	pub fn into_stream(self) -> S {
		self.stream
	}

	/// Returns an iterator over incoming messages.
	///
	///```no_run
	///# extern crate websocket;
	///# fn main() {
	///use websocket::{ClientBuilder, Message};
	///
	///let mut client = ClientBuilder::new("ws://127.0.0.1:1234").unwrap()
	///                     .connect(None).unwrap();
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
	///use websocket::{ClientBuilder, Message};
	///
	///let mut client = ClientBuilder::new("ws://127.0.0.1:1234").unwrap()
	///                     .connect_insecure().unwrap();
	///
	///let (mut receiver, mut sender) = client.split().unwrap();
	///
	///for message in receiver.incoming_messages() {
	///    let message: Message = message.unwrap();
	///    // Echo the message back
	///    sender.send_message(&message).unwrap();
	///}
	///# }
	///```
	pub fn incoming_messages<'a, M, D>(&'a mut self) -> MessageIterator<'a, Receiver, D, M, S>
		where M: ws::Message<'a, D>,
		      D: DataFrameable
	{
		self.receiver.incoming_messages(&mut self.stream)
	}
}

impl<S> Client<S>
    where S: Splittable + Stream
{
	/// Split this client into its constituent Sender and Receiver pair.
	///
	/// This allows the Sender and Receiver to be sent to different threads.
	///
	///```no_run
	///# extern crate websocket;
	///# fn main() {
	///use std::thread;
	///use websocket::{ClientBuilder, Message};
	///
	///let mut client = ClientBuilder::new("ws://127.0.0.1:1234").unwrap()
	///                     .connect_insecure().unwrap();
	///
	///let (mut receiver, mut sender) = client.split().unwrap();
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
	pub fn split
		(self,)
		 -> IoResult<(Reader<<S as Splittable>::Reader>, Writer<<S as Splittable>::Writer>)> {
		let (read, write) = try!(self.stream.split());
		Ok((Reader {
		        stream: read,
		        receiver: self.receiver,
		    },
		    Writer {
		        stream: write,
		        sender: self.sender,
		    }))
	}
}
