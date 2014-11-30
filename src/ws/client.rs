use super::handshake::request::{WebSocketRequest, ReadWebSocketRequest, WriteWebSocketRequest};
use super::handshake::response::{WebSocketResponse, ReadWebSocketResponse, WriteWebSocketResponse};
use super::message::send::{WebSocketSender, new_sender};
use super::message::receive::{WebSocketReceiver, new_receiver};
use std::io::net::tcp::TcpStream;
use std::io::net::ip::SocketAddr;
use std::io::{IoResult, IoError, IoErrorKind};
use std::clone::Clone;

/// Represents a WebSocket client.
/// To use WebSocketClient, you must create one using either WebSocketClient::connect(),
/// which is used for writing clients, or WebSocketClient::from_stream(), which creates
/// a WebSocketClient from a TcpStream (typically used in a server).
///
/// An example client application:
/// 
/// ```no_run
/// use websocket::WebSocketClient;
/// use websocket::handshake::WebSocketRequest;
/// 
/// let request = WebSocketRequest::new("ws://127.0.0.1:1234", "myProtocol").unwrap();
/// let key = request.key().unwrap();
/// let mut client = WebSocketClient::connect(&request).unwrap();
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
pub struct WebSocketClient {
	stream: TcpStream,
	mask: bool,
}

impl WebSocketClient {
	/// Connect to the WebSocket server using the given request. Use WebSocketRequest::new() to create a request for use
	/// with this function.
	pub fn connect(request: &WebSocketRequest) -> IoResult<WebSocketClient> {
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
		try!(stream.write_websocket_request(request));
		
		Ok(WebSocketClient{
			stream: stream,
			mask: true,
		})
	}
	
	/// Creates a new WebSocketClient from a given TcpStream.
	/// The mask parameter determines whether or not messages send to the remote endpoint will be masked.
	/// If the client is connecting to a remote endpoint, set mask to true. If the client is the remote
	/// endpoint (and therefore, the server is the local endpoint), set mask to false.
	pub fn from_stream(stream: TcpStream, mask: bool) -> WebSocketClient {
		WebSocketClient {
			stream: stream,
			mask: mask,
		}
	}
	
	/// Returns a copy of the underlying TcpStream for this WebSocketClient.
	pub fn stream(&self) -> TcpStream {
		self.stream.clone()
	}
	
	/// Returns the socket address of the remote peer of this TCP connection.
	pub fn peer_name(&mut self) -> IoResult<SocketAddr> {
		self.stream.peer_name()
	}

	/// Returns the socket address of the local half of this TCP connection.
	pub fn socket_name(&mut self) -> IoResult<SocketAddr> {
		self.stream.socket_name()
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
	
	/// Closes the sender for this WebSocketClient.
	/// This method will close the message sending portion of this client, causing all pending and future sends to immediately return with an error.
	/// This affects all WebSocketSenders for the client, and any copies of the underlying stream will be unable to write.
	///
	/// Note that you should send a WebSocketMessage:Close message to the remote endpoint before calling this method.
	pub fn close_send(&mut self) -> IoResult<()> {
		self.stream.close_write()
	}
	
	/// Returns a WebSocketReceiver from this client. Used to receive data from the remote endpoint,
	/// that is, from the server if WebSocketClient::connect() has been used, or from this client otherwise.
	pub fn receiver(&self) -> WebSocketReceiver {
		new_receiver(self.stream.clone())
	}
	
	/// Closes the receiver for this WebSocketClient.
	/// This method will close the message receiving portion of this client, causing all pending and future receives to immediately return with an error.
	/// This affects all WebSocketReceivers for the client, and any copies of the underlying stream will be unable to read.
	///
	/// Note that you should send a WebSocketMessage:Close message to the remote endpoint before calling this method.
	pub fn close_receive(&mut self) -> IoResult<()> {
		self.stream.close_read()
	}
}

impl Clone for WebSocketClient {
	fn clone(&self) -> WebSocketClient {
		WebSocketClient {
			stream: self.stream.clone(),
			mask: self.mask,
		}
	}
}