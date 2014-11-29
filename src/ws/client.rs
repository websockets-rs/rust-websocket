use super::handshake::request::{WebSocketRequest, ReadWebSocketRequest, WriteWebSocketRequest};
use super::handshake::response::{WebSocketResponse, ReadWebSocketResponse, WriteWebSocketResponse};
use super::message::send::new_sender;
use super::message::receive::new_receiver;
use super::message::{WebSocketSender, WebSocketReceiver};
use std::io::net::tcp::TcpStream;
use std::io::{IoResult, IoError, IoErrorKind};
use std::option::Option;
use std::clone::Clone;

/// Represents a WebSocket client.
pub struct WebSocketClient {
	stream: TcpStream,
	request: Option<WebSocketRequest>,
	response: Option<WebSocketResponse>,
	mask: bool,
}

impl WebSocketClient {
	/// Connect to the WebSocket server using the given request. Use WebSocketRequest::new() to create a request for use
	/// with this function.
	pub fn connect(request: WebSocketRequest) -> IoResult<WebSocketClient> {
		let host = try!(request.headers.get("Host").ok_or(
			IoError {
				kind: IoErrorKind::InvalidInput,
				desc: "No host specified",
				detail: None,
			}
		));
		//Connect to the server
		let mut stream = try!(TcpStream::connect(host.as_slice()));
		//Send the opening handshake
		try!(stream.write_websocket_request(&request));
		//Get a response
		let response = try!(stream.read_websocket_response());
		Ok(WebSocketClient{
			stream: stream,
			request: Some(request),
			response: Some(response),
			mask: true,
		})
	}
	
	/// Creates a new WebSocketClient from a given TcpStream.
	/// The mask parameter determines whether or not messages send to the remote endpoint will be masked.
	/// If the client is connecting to a remote endpoint, set mask to true. If the client is the remote
	/// endpoint (and therefore, the server is the local endpoint), set mask to false.
	/// Typically, WebSocketClient::create() will be used instead of this function.
	pub fn from_stream(stream: TcpStream, mask: bool) -> WebSocketClient {
		WebSocketClient {
			stream: stream,
			request: None,
			response: None,
			mask: mask,
		}
	}
	
	/// Reads a request from this client. Only to be used if the server is the local endpoint and
	/// the client is the remote endpoint.
	pub fn receive_handshake_request(&mut self) -> IoResult<WebSocketRequest> {
		self.stream.read_websocket_request()
	}
	
	/// Sends the specified WebSocketResponse to this client. Only to be used if the server is
	/// the local endpoint and the client is the remote endpoint.
	pub fn send_handshake_response(&mut self, response: WebSocketResponse) -> IoResult<()> {
		self.stream.write_websocket_response(&response)
	}
	
	/// Returns a WebSocketSender from this client. Used to transmit data to the remote endpoint,
	/// that is, to the server if WebSocketClient::connect() has been used, or to this client otherwise.
	pub fn sender(&self) -> WebSocketSender {
		new_sender(self.stream.clone(), self.mask)
	}
	
	/// Returns a WebSocketReceiver from this client. Used to receive data from the remote endpoint,
	/// that is, from the server if WebSocketClient::connect() has been used, or from this client otherwise.
	pub fn receiver(&self) -> WebSocketReceiver {
		new_receiver(self.stream.clone())
	}
}

impl Clone for WebSocketClient {
	fn clone(&self) -> WebSocketClient {
		WebSocketClient {
			stream: self.stream.clone(),
			request: self.request.clone(),
			response: self.response.clone(),
			mask: self.mask,
		}
	}
}