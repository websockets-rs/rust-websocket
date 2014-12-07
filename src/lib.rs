#![crate_type = "lib"]
#![crate_name = "websocket"]

//! Rust-WebSocket is a WebSocket (RFC6455) library written in Rust.
//! Rust-WebSocket attempts to provide a framework for WebSocket connections (both clients and servers).
//! The library is currently in an experimental state, but can work as a simple WebSocket server or client,
//! with the capability to send and receive fragmented messages.
//! 
//! Rust-WebSocket does not provide a 'listener' object, since the WebSocket protocol utilises a TcpStream.
//! To implement a WebSocket server, use a normal TcpListener, and the WebSocketClient::new() function
//! on an accepted stream. You will then need to read the handshake from the client and send a response.
//! 
//! ```no_run
//! use std::io::TcpListener;
//! use std::io::{Listener, Acceptor};
//! use websocket::WebSocketClient;
//! use websocket::handshake::WebSocketResponse;
//! 
//! let listener = TcpListener::bind("127.0.0.1:1234");
//! let mut acceptor = listener.listen();
//! 
//! for stream in acceptor.incoming() {
//! 	match stream {
//! 		Ok(stream) => {
//! 			// Spawn a new task for each connection to run in parallel
//! 			spawn(proc() {
//! 				// Get a WebSocketClient from this stream
//! 				// The mask argument is false as messages sent by the server are always unmasked
//! 				let mut client = WebSocketClient::new(stream, false); 
//! 				
//! 				// Read the handshake from the client
//! 				let request = client.receive_handshake_request().unwrap();
//! 				
//! 				// Get the Sec-WebSocket-Key from the request
//! 				let key = request.key().unwrap();
//! 				
//! 				// Form a response from the key
//!					/* In this example, we don't deal with the requested Sec-WebSocket-Protocol.
//!					   Because of this, however, we need a type annotation, which would 
//!					   not usually be required. */
//! 				let response = WebSocketResponse::new::<String>(key.as_slice(), None);
//! 				
//! 				// Send the response to the client
//! 				let _ = client.send_handshake_response(response);
//! 				
//! 				// Now we can send and receive messages
//! 				let receiver = client.receiver();
//! 				let mut sender = client.sender();
//! 				
//! 				// ...
//! 			});
//! 		}
//! 		_ => { /* A connection error occurred */ }
//! 	}
//! }
//! ```
#![feature(phase)]
extern crate regex;
extern crate url;
extern crate sha1;
extern crate serialize;

pub use self::ws::client::WebSocketClient;

/// Structs for manipulation of HTTP headers. Used in conjunction with 
/// WebSocketRequest and WebSocketResponse.
pub mod headers {
	pub use util::header::{HeaderCollection, Headers};
	pub use url::ParseResult;
	pub use url::ParseError;
}

/// Structs for WebSocket handshake requests and responses
pub mod handshake {
	pub use ws::handshake::request::WebSocketRequest;
	pub use ws::handshake::response::WebSocketResponse;
}

/// Structs for WebSocket messages and the transmission of messages
pub mod message {
	pub use ws::message::WebSocketMessage;
	pub use ws::message::send::{WebSocketSender, WebSocketFragmentSerializer};
	pub use ws::message::receive::{WebSocketReceiver, IncomingMessages};
}

mod ws;
mod util;