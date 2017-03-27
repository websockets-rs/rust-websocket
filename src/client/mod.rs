//! Contains the WebSocket client.
use std::net::TcpStream;
use std::marker::PhantomData;
use std::io::Result as IoResult;
use std::io::{
    Read,
    Write,
};
use std::ops::Deref;

use ws;
use ws::sender::Sender as SenderTrait;
use ws::util::url::ToWebSocketUrlComponents;
use ws::receiver::{
    DataFrameIterator,
    MessageIterator,
};
use ws::receiver::Receiver as ReceiverTrait;
use result::WebSocketResult;
use stream::{
	  AsTcpStream,
	  Stream,
    Splittable,
    Shutdown,
};
use dataframe::DataFrame;
use ws::dataframe::DataFrame as DataFrameable;

use openssl::ssl::{SslContext, SslMethod, SslStream};

pub use self::request::Request;
pub use self::response::Response;

use sender::Sender;
pub use sender::Writer;

use receiver::Receiver;
pub use receiver::Reader;

pub mod request;
pub mod response;

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
// TODO: add back client.split()

pub struct ClientBuilder<S>
    where S: Stream,
{
    stream: S,
}

// impl Client<TcpStream> {
// 	/// Connects to the given ws:// URL and return a Request to be sent.
// 	///
// 	/// If you would like to use a secure connection (wss://), please use `connect_secure`.
// 	///
// 	/// A connection is established, however the request is not sent to
// 	/// the server until a call to ```send()```.
// 	pub fn connect<C>(components: C) -> WebSocketResult<Request<TcpStream, TcpStream>>
// 	where C: ToWebSocketUrlComponents,
// 	{
// 		let (host, resource_name, secure) = try!(components.to_components());
// 		let stream = TcpStream::connect((&host.hostname[..], host.port.unwrap_or(80)));
// 		let stream = try!(stream);
// 		Request::new((host, resource_name, secure), try!(stream.split()))
// 	}
// }

// impl Client<SslStream<TcpStream>> {
// 	/// Connects to the specified wss:// URL using the given SSL context.
// 	///
// 	/// If you would like to use an insecure connection (ws://), please use `connect`.
// 	///
// 	/// A connection is established, however the request is not sent to
// 	/// the server until a call to ```send()```.
// 	pub fn connect_secure<C>(components: C, context: Option<&SslContext>) -> WebSocketResult<Request<SslStream<TcpStream>, SslStream<TcpStream>>>
// 	where C: ToWebSocketUrlComponents,
// 	{
// 		let (host, resource_name, secure) = try!(components.to_components());

// 		let stream = TcpStream::connect((&host.hostname[..], host.port.unwrap_or(443)));
// 		let stream = try!(stream);
// 		let sslstream = if let Some(c) = context {
// 			SslStream::connect(c, stream)
// 		} else {
// 			let context = try!(SslContext::new(SslMethod::Tlsv1));
// 			SslStream::connect(&context, stream)
// 		};
// 		let sslstream = try!(sslstream);

// 		Request::new((host, resource_name, secure), try!(sslstream.split()))
// 	}
// }

// // TODO: look at how to get hyper to give you a stream then maybe remove this
// impl Client<Box<AsTcpStream>> {
// 	pub fn connect_agnostic<C>(components: C, ssl_context: Option<&SslContext>) -> WebSocketResult<Request<Box<AsTcpStream>, Box<AsTcpStream>>>
// 	where C: ToWebSocketUrlComponents
// 	{
// 		let (host, resource_name, secure) = try!(components.to_components());
// 		let port = match host.port {
// 			Some(p) => p,
// 			None => if secure {
// 				443
// 			} else {
// 				80
// 			},
// 		};
// 		let tcp_stream = try!(TcpStream::connect((&host.hostname[..], port)));

// 		let stream: Box<AsTcpStream> = if secure {
// 			if let Some(c) = ssl_context {
// 				Box::new(try!(SslStream::connect(c, tcp_stream)))
// 			} else {
// 				let context = try!(SslContext::new(SslMethod::Tlsv1));
// 				Box::new(try!(SslStream::connect(&context, tcp_stream)))
// 			}
// 		} else {
// 			Box::new(tcp_stream)
// 		};

// 		let (read, write) = (try!(stream.duplicate()), stream);

// 		Request::new((host, resource_name, secure), (read, write))
// 	}
// }

impl<S> Client<S>
    where S: AsTcpStream + Stream,
{
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

	  /// Shuts down the client connection, will cause all pending and future IO to
	  /// return immediately with an appropriate value.
	  pub fn shutdown(&self) -> IoResult<()> {
		    self.stream.as_tcp().shutdown(Shutdown::Both)
	  }

    // TODO: add net2 set_nonblocking and stuff
}

impl<S> Client<S>
    where S: Stream,
{
	  /// Crtes a Client from the given Sender and Receiver.
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
	      where D: DataFrameable {
		    self.sender.send_dataframe(self.stream.writer(), dataframe)
	  }

	  /// Sends a single message to the remote endpoint.
	  pub fn send_message<'m, M, D>(&mut self, message: &'m M) -> WebSocketResult<()>
	      where M: ws::Message<'m, D>, D: DataFrameable {
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
