use super::handshake::request::{WebSocketRequest, ReadWebSocketRequest, WriteWebSocketRequest};
use super::handshake::response::{WebSocketResponse, ReadWebSocketResponse, WriteWebSocketResponse};
use super::message::send::{WebSocketSender, new_sender};
use super::message::receive::{WebSocketReceiver, new_receiver};
use std::io::{Stream, IoResult};
use std::clone::Clone;

/// Represents a WebSocket client.
/// To use WebSocketClient, you must create one using WebSocketClient::new().
/// For a client, you can use TcpStream::connect() to connect to the server, then call
/// WebSocketClient::new() passing the resultant stream.
/// 
/// ```no_run
/// use std::io::TcpStream;
/// use websocket::{WebSocketClient, WebSocketClientMode};
/// use websocket::handshake::WebSocketRequest;
/// 
/// let request = WebSocketRequest::new("ws://127.0.0.1:1234", ["myProtocol"].as_slice()).unwrap();
/// let key = request.key().unwrap();
///
/// // We can get the hostname from the request
/// let stream = TcpStream::connect(request.host().unwrap().as_slice()).unwrap();
///
/// let mut client = WebSocketClient::new(stream, WebSocketClientMode::RemoteServer);
/// let _ = client.send_handshake_request(&request);
/// let response = client.receive_handshake_response().unwrap();
/// 
/// if !response.is_successful(key) {
/// 	// Handshake failed!
/// }
/// 
/// // Now we can send and receive messages
/// let receiver = client.receiver();
/// let mut sender = client.sender();
/// 
/// // ...
/// ```
pub struct WebSocketClient<S: Stream + Clone> {
	stream: S,
	mask: bool,
}

impl<S: Stream + Clone> WebSocketClient<S> {
	/// Creates a new WebSocketClient from a given cloneable stream.
	/// The mask parameter determines whether or not messages send to the remote endpoint will be masked.
	/// If the client is connecting to a remote endpoint, set mask to true. If the client is the remote
	/// endpoint (and therefore, the server is the local endpoint), set mask to false.
	/// 
	/// Nothing is sent to or read from the stream during the conversion.
	pub fn new(stream: S, mode: WebSocketClientMode) -> WebSocketClient<S> {
		WebSocketClient {
			stream: stream,
			mask: match mode {
				WebSocketClientMode::RemoteClient => { false }
				WebSocketClientMode::RemoteServer => { true }
			},
		}
	}
	
	/// Returns a copy of the underlying S for this WebSocketClient.
	/// Note that writing to this stream will likely invalidate the WebSocket data stream.
	pub fn stream(&self) -> S {
		self.stream.clone()
	}
	
	/// Reads a request from this client. Only to be used if the server is the local endpoint and
	/// the client is the remote endpoint.
	pub fn receive_handshake_request(&mut self) -> IoResult<WebSocketRequest> {
		self.stream.read_websocket_request()
	}
	
	/// Reads a response that was sent to this client. Only to be used if the server is the remote
	/// endpoint and the client is the local endpoint.
	pub fn receive_handshake_response(&mut self) -> IoResult<WebSocketResponse> {
		self.stream.read_websocket_response()
	}
	
	/// Sends the specified WebSocketRequest to the remote endpoint. Only to be used if the server is
	/// the remote endpoint and the client is the local endpoint.
	pub fn send_handshake_request(&mut self, request: &WebSocketRequest) -> IoResult<()> {
		self.stream.write_websocket_request(request)
	}
	
	/// Sends the specified WebSocketResponse to this client. Only to be used if the server is
	/// the local endpoint and the client is the remote endpoint.
	pub fn send_handshake_response(&mut self, response: &WebSocketResponse) -> IoResult<()> {
		self.stream.write_websocket_response(response)
	}
	
	/// Returns a WebSocketSender from this client. Used to transmit data to the remote endpoint,
	/// that is, to the server if WebSocketClient::connect() has been used, or to this client otherwise.
	pub fn sender(&self) -> WebSocketSender<S> {
		new_sender(self.stream.clone(), self.mask)
	}
	
	/// Returns a WebSocketReceiver from this client. Used to receive data from the remote endpoint,
	/// that is, from the server if WebSocketClient::connect() has been used, or from this client otherwise.
	pub fn receiver(&self) -> WebSocketReceiver<S> {
		new_receiver(self.stream.clone())
	}
}

impl<S: Stream + Clone> Clone for WebSocketClient<S> {
	fn clone(&self) -> WebSocketClient<S> {
		WebSocketClient {
			stream: self.stream.clone(),
			mask: self.mask,
		}
	}
}

pub enum WebSocketClientMode {
	RemoteClient,
	RemoteServer,
}