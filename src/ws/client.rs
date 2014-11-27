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
	
	/// Gets the request associated with this client (if any)
	pub fn request(&self) -> Option<WebSocketRequest> {
		self.request.clone()
	}
	
	/// Gets the response associated with this client (if any)
	pub fn response(&self) -> Option<WebSocketResponse> {
		self.response.clone()
	}
	
	/// Sends the specified WebSocketResponse to this client - only to be used from a server.
	/// Do not use if you've connected to a server using the WebSocketClient::connect() function.
	pub fn send_handshake_response(&mut self, response: WebSocketResponse) -> IoResult<()> {
		try!(self.stream.write_websocket_response(&response));
		self.response = Some(response);
		Ok(())
	}
	
	/// Returns a WebSocketSender from this client. Use to transmit data to this client (for a server)
	/// or to the server (if this is a client using the WebSocketClient::connect() function).
	pub fn sender(&self) -> WebSocketSender {
		new_sender(self.stream.clone(), self.mask)
	}
	
	/// Returns a WebSocketReceiver from this client. Use to receive messages from this client (for a server)
	/// or to receive messages from the server (if this is a client using the WebSocketClient::connect() function).
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

pub fn serverside_client(mut stream: TcpStream) -> IoResult<WebSocketClient> {
	let request = try!(stream.read_websocket_request());
	Ok(WebSocketClient {
		stream: stream,
		request: Some(request),
		response: None,
		mask: false,
	})
}