//! Structs for dealing with WebSocket clients
#![unstable]

use dataframe::{DataFrameSender, DataFrameReceiver, DataFrameConverter};
use dataframe::WebSocketDataFrame;
use message::WebSocketMessaging;
use common::{WebSocketResult, WebSocketError, DataAvailable};
use std::path::BytesContainer;
use std::sync::{Arc, Mutex};

pub use self::incoming::{IncomingDataFrames, IncomingMessages};
pub use self::fragment::{TextFragmentSender, BinaryFragmentSender};

use dataframe::{WebSocketSender, WebSocketReceiver, WebSocketConverter};
use common::{WebSocketStream, Local, Remote};
use WebSocketMessage;

pub mod incoming;
pub mod fragment;

/// The most common local WebSocketClient type, provided for convenience.
pub type WebSocketLocalClient = WebSocketClient<WebSocketSender<WebSocketStream, Local>, WebSocketReceiver<WebSocketStream, Local>, WebSocketConverter<WebSocketMessage>>;
/// The most common remote WebSocketClient type, provided for convenience.
pub type WebSocketRemoteClient = WebSocketClient<WebSocketSender<WebSocketStream, Remote>, WebSocketReceiver<WebSocketStream, Remote>, WebSocketConverter<WebSocketMessage>>;

/// Represents a WebSocketClient which connects to a WebSocketServer.
/// 
/// See the main library documentation for how to obtain a ```WebSocketClient```.
/// The most commonly used methods are ```send_message()``` and ```recv_message()```.
///
/// Note that you can ```clone()``` a WebSocketClient, and that clone will represent
/// the same connection. The cloned clients may be captured into another thread so
/// concurrent sending/receiving can occur. A locking mechanism is used to ensure that
/// whole dataframes/messages are sent or received by the client.
pub struct WebSocketClient<S, R, C> {
	sender: Arc<Mutex<S>>,
	receiver: Arc<Mutex<(R, C)>>,
}

unsafe impl<S: Send, R: Send, C: Send> Send for WebSocketClient<S, R, C> {}

#[old_impl_check]
impl<S: DataFrameSender<W>, R: DataFrameReceiver<E>, C: DataFrameConverter<M>, E: Reader + Send, W: Writer + Send, M: WebSocketMessaging> WebSocketClient<S, R, C> {
	/// Create a WebSocketClient from the specified DataFrameSender and DataFrameReceiver.
	/// Not required for normal usage (used internally by ```WebSocketResponse```).
	pub fn new(sender: S, receiver: R, converter: C) -> WebSocketClient<S, R, C> {
		WebSocketClient {
			sender: Arc::new(Mutex::new(sender)),
			receiver: Arc::new(Mutex::new((receiver, converter))),
		}
	}
	
	/// Sends a single WebSocketDataFrame. Blocks until the message has been sent.
	#[stable]
	pub fn send_dataframe(&mut self, dataframe: &WebSocketDataFrame) -> WebSocketResult<()> {
		let mut sender = self.sender.lock().unwrap();
		sender.send_dataframe(dataframe)
	}
	
	/// Receives a single WebSocketDataFrame - may corrupt messages received from recv_message(),
	/// so do not use both at the same time (ie. either use recv_dataframe() exclusively or use
	/// recv_message() exclusively).
	#[stable]
	pub fn recv_dataframe(&mut self) -> WebSocketResult<WebSocketDataFrame> {
		let mut receiver = self.receiver.lock().unwrap();
		receiver.0.recv_dataframe()
	}

	/// Returns an iterator over incoming data frames.
	#[stable]
	pub fn incoming_dataframes(&mut self) -> IncomingDataFrames<S, R, C> {
		IncomingDataFrames::new(self)
	}
	
	/// Returns an iterator over incoming messages.
	/// 
	/// The iterator always returns Some(), and each iteration will block until a message is received.
	/// 
	/// ```no_run
	///# extern crate url;
	///# extern crate websocket;
	///# fn main() {
	///# use websocket::{WebSocketRequest, WebSocketMessage};
	///# use url::Url;
	///# let url = Url::parse("ws://127.0.0.1:1234").unwrap();
	///# let request = WebSocketRequest::connect(url).unwrap();
	///# let response = request.send().unwrap();
	///# let mut client = response.begin();
	///for message in client.incoming_messages() {
	///    match message.unwrap() {
	///        WebSocketMessage::Text(text) => { println!("Text: {}", text); },
	///        WebSocketMessage::Binary(data) => { println!("Binary data received"); },
	///        _ => { }
	///    }
	///}
	///# }
	/// ```
	pub fn incoming_messages(&mut self) -> IncomingMessages<S, R, C> {
		IncomingMessages::new(self)
	}
	
	/// Sends a fragmented text message by returning a TextFragmentSender
	///
	/// ```no_run
	///# extern crate url;
	///# extern crate websocket;
	///# fn main() {
	///# use websocket::WebSocketRequest;
	///# use url::Url;
	///# let url = Url::parse("ws://127.0.0.1:1234").unwrap();
	///# let request = WebSocketRequest::connect(url).unwrap();
	///# let response = request.send().unwrap();
	///# let mut client = response.begin();
	///let mut fragment_sender = client.frag_send_text("This").unwrap();
	///let _ = fragment_sender.send("is ");
	///let _ = fragment_sender.send("a ");
	///let _ = fragment_sender.send("fragmented ");
	///let _ = fragment_sender.finish("message.");
	///# }
	/// ```
	pub fn frag_send_text<'a, T: ToString>(&'a mut self, text: T) -> WebSocketResult<TextFragmentSender<'a, S, W>> {
		TextFragmentSender::new(self.sender.lock().unwrap(), text)
	}
	
	/// Sends a fragmented binary message by returning a BinaryFragmentSender
	///
	/// ```no_run
	///# extern crate url;
	///# extern crate websocket;
	///# fn main() {
	///# use websocket::WebSocketRequest;
	///# use url::Url;
	///# let url = Url::parse("ws://127.0.0.1:1234").unwrap();
	///# let request = WebSocketRequest::connect(url).unwrap();
	///# let response = request.send().unwrap();
	///# let mut client = response.begin();
	///let mut fragment_sender = client.frag_send_bytes("ascii_bytes").unwrap();
	///let _ = fragment_sender.send([100u8; 4].as_slice());
	///let _ = fragment_sender.finish(vec![4u8, 2, 68, 24]);
	///# }
	/// ```
	pub fn frag_send_bytes<'a, T: BytesContainer>(&'a mut self, data: T) -> WebSocketResult<BinaryFragmentSender<'a, S, W>> {
		BinaryFragmentSender::new(self.sender.lock().unwrap(), data)
	}
	
	/// Sends a WebSocketMessage. Blocks until the message has been sent.
	/// 
	/// ```no_run
	///# extern crate url;
	///# extern crate websocket;
	///# fn main() {
	///# use websocket::{WebSocketRequest, WebSocketMessage};
	///# use url::Url;
	///# let url = Url::parse("ws://127.0.0.1:1234").unwrap();
	///# let request = WebSocketRequest::connect(url).unwrap();
	///# let response = request.send().unwrap();
	///# let mut client = response.begin();
	///let message = WebSocketMessage::Text("Hello, server!".to_string());
	///let _ = client.send_message(message);
	///# }
	/// ```
	pub fn send_message(&mut self, message: M) -> WebSocketResult<()> {
		let dataframe = try!(message.into_dataframe());
		self.send_dataframe(&dataframe)
	}
	
	/// Receives a WebSocketMessage. Blocks until a full message is received.
	/// 
	/// ```no_run
	///# extern crate url;
	///# extern crate websocket;
	///# fn main() {
	///# use websocket::{WebSocketRequest, WebSocketMessage};
	///# use url::Url;
	///# let url = Url::parse("ws://127.0.0.1:1234").unwrap();
	///# let request = WebSocketRequest::connect(url).unwrap();
	///# let response = request.send().unwrap();
	///# let mut client = response.begin();
	///let message = client.recv_message().unwrap();
	///match message {
	///    WebSocketMessage::Text(text) => { println!("Text: {}", text); },
	///    WebSocketMessage::Binary(data) => { println!("Binary data received"); },
	///    _ => { }
	///}
	///# }
	/// ```
	pub fn recv_message(&mut self) -> WebSocketResult<M> {
		let mut receiver = self.receiver.lock().unwrap();
		loop {
			let dataframe = try!(receiver.0.recv_dataframe());
			try!(receiver.1.push(dataframe));
			match receiver.1.pop() {
				Some(message) => { return Ok(message); }
				None => { }
			}
		}
	}
}

#[old_impl_check]
impl<S: DataFrameSender<W>, R: DataFrameReceiver<E> + DataAvailable, C: DataFrameConverter<M>, E: Reader + DataAvailable + Send, W: Writer + Send, M: WebSocketMessaging> WebSocketClient<S, R, C> {
	/// Try to receive a data frame.
	///
	/// If no data is available, the functino returns immediately with the error NoDataAvailable.
	/// If there is data available, the function will block until the whole data frame is received.
	pub fn try_recv_dataframe(&mut self) -> WebSocketResult<WebSocketDataFrame> {
		let mut receiver = self.receiver.lock().unwrap();
		if receiver.0.data_available() {
			receiver.0.recv_dataframe()
		}
		else {
			Err(WebSocketError::NoDataAvailable)
		}
	}
	/// Try to receive a message.
	///
	/// If no data is available, the function returns immediately with the error NoDataAvailable.
	/// If there is data available, the function will block until the whole message is received.
	pub fn try_recv_message(&mut self) -> WebSocketResult<M> {
		let mut receiver = self.receiver.lock().unwrap();
		if receiver.0.data_available() {
			loop {
				let dataframe = try!(receiver.0.recv_dataframe());
				try!(receiver.1.push(dataframe));
				match receiver.1.pop() {
					Some(message) => { return Ok(message); }
					None => { }
				}
			}
		}
		else {
			Err(WebSocketError::NoDataAvailable)
		}
	}
}

#[old_impl_check]
impl<S, R, C> Clone for WebSocketClient<S, R, C> {
	/// Clone this WebSocketClient, allowing for concurrent operations on a single stream.
	/// 
	/// All cloned clients refer to the same underlying stream. Simultaneous reads will not
	/// return the same data; the first read will obtain one WebSocketMessage/WebSocketDataFrame,	
	/// the second will obtain the next WebSocketMessage/WebSocketDataFrame, etc.
	#[stable]
	fn clone(&self) -> WebSocketClient<S, R, C> {
		WebSocketClient {
			sender: self.sender.clone(),
			receiver: self.receiver.clone(),
		}
	}
}