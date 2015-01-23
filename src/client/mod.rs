//! Contains the WebSocket client.

use std::io::TcpStream;

use ws;
use ws::util::url::url_to_host;
use ws::receiver::{DataFrameIterator, MessageIterator};
use result::{WebSocketResult, WebSocketError};
use stream::WebSocketStream;
use dataframe::DataFrame;

use url::Url;

use openssl::ssl::{SslContext, SslMethod, SslStream};

pub use self::request::Request;
pub use self::response::Response;

pub use self::sender::Sender;
pub use self::receiver::Receiver;

pub mod sender;
pub mod receiver;
pub mod request;
pub mod response;

/// Represents a WebSocket client, which can send and receive messages/data frames.
///
/// `D` is the data frame type, `S` is the type implementing `Sender<D>` and `R`
/// is the type implementing `Receiver<D>`.
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
///let message = Message::Text("Hello, World!".to_string());
///client.send_message(message).unwrap(); // Send message
///# }
///```
pub struct Client<D, S, R> {
	sender: S,
	receiver: R
}

impl Client<DataFrame, Sender<WebSocketStream>, Receiver<WebSocketStream>> {
	/// Connects to the given ws:// or wss:// URL and return a Request to be sent.
	///
	/// A connection is established, however the request is not sent to
	/// the server until a call to ```send()```.
	pub fn connect(url: Url) -> WebSocketResult<Request<WebSocketStream, WebSocketStream>> {
		let context = try!(SslContext::new(SslMethod::Tlsv1));
		Client::connect_ssl_context(url, &context)
	}
	/// Connects to the specified wss:// URL using the given SSL context.
	///
	/// If a ws:// URL is supplied, a normal, non-secure connection is established
	/// and the context parameter is ignored.
	///
	/// A connection is established, however the request is not sent to
	/// the server until a call to ```send()```.
	pub fn connect_ssl_context(url: Url, context: &SslContext) -> WebSocketResult<Request<WebSocketStream, WebSocketStream>> {
		let host = try!(url_to_host(&url).ok_or(
			WebSocketError::RequestError("Could not get socket address from URL".to_string())
		));
		
		let connection = try!(TcpStream::connect(&(
			host.hostname + ":" + 
			&host.port.unwrap().to_string()[]
		)[]));
		
		let stream = match &url.scheme[] {
			"ws" => {
				WebSocketStream::Tcp(connection)
			}
			"wss" => {
				let sslstream = try!(SslStream::new(context, connection));
				WebSocketStream::Ssl(sslstream)
			}
			_ => { return Err(WebSocketError::RequestError("URI scheme not supported".to_string())); }
		};
		
		Request::new(url, stream.clone(), stream)
	}
}

impl<D, S: ws::Sender<D>, R: ws::Receiver<D>> Client<D, S, R> {
	/// Creates a Client from the given Sender and Receiver.
	///
	/// Essentially the opposite of `Client.split()`.
	pub fn new(sender: S, receiver: R) -> Client<D, S, R> {
		Client {
			sender: sender,
			receiver: receiver
		}
	}
	/// Sends a single data frame to the remote endpoint.
	pub fn send_dataframe(&mut self, dataframe: D) -> WebSocketResult<()> {
		self.sender.send_dataframe(dataframe)
	}
	/// Sends a single message to the remote endpoint.
	pub fn send_message<M>(&mut self, message: M) -> WebSocketResult<()> 
		where M: ws::Message<D>, <M as ws::Message<D>>::DataFrameIterator: Iterator<Item = D> {
		
		self.sender.send_message(message)
	}
	/// Reads a single data frame from the remote endpoint.
	pub fn recv_dataframe(&mut self) -> WebSocketResult<D> {
		self.receiver.recv_dataframe()
	}
	/// Returns an iterator over incoming data frames.
	pub fn incoming_dataframes<'a>(&'a mut self) -> DataFrameIterator<'a, R, D> {
		self.receiver.incoming_dataframes()
	}
	/// Reads a single message from this receiver.
	pub fn recv_message(&mut self) -> WebSocketResult<<R as ws::Receiver<D>>::Message> {
		self.receiver.recv_message()
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
	///    println!("Recv: {:?}", message.unwrap());
	///}
	///# }
	///```
	pub fn incoming_messages<'a>(&'a mut self) -> MessageIterator<'a, R, D> {
		self.receiver.incoming_messages()
	}
	/// Returns a reference to the underlying Sender.
	pub fn get_sender(&self) -> &S {
		&self.sender
	}
	/// Returns a reference to the underlying Receiver.
	pub fn get_reciever(&self) -> &R {
		&self.receiver
	}
	/// Returns a mutable reference to the underlying Sender.
	pub fn get_mut_sender(&mut self) -> &mut S {
		&mut self.sender
	}
	/// Returns a mutable reference to the underlying Receiver.
	pub fn get_mut_reciever(&mut self) -> &mut R {
		&mut self.receiver
	}
	/// Split this client into its constituent Sender and Receiver pair.
	///
	/// This allows the Sender and Receiver to be sent to different threads.
	///
	///```no_run
	///# extern crate websocket;
	///# fn main() {
	///use websocket::{Client, Message, Sender, Receiver};
	///use std::thread::Thread;
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
	///Thread::spawn(move || {
	///    for message in receiver.incoming_messages() {
	///        println!("Recv: {:?}", message.unwrap());
	///    }
	///});
	///
	///let message = Message::Text("Hello, World!".to_string());
	///sender.send_message(message).unwrap();
	///# }
	///```
	pub fn split(self) -> (S, R) {
		(self.sender, self.receiver)
	}
}