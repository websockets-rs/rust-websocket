extern crate websocket;

use std::os;
use std::sync::TaskPool;
use std::thread::Thread;
use std::io::{Listener, Acceptor};
use std::io::IoResult;
use websocket::{WebSocketServer, WebSocketMessage};
use websocket::header::WebSocketProtocol;
use websocket::common::WebSocketError;
use websocket::handshake::request::WebSocketInboundRequest;
use websocket::client::WebSocketRemoteClient;

fn main() {
	// Bind to the socket
	let server = WebSocketServer::bind("127.0.0.1:2794").unwrap();
	// Listen for connections
	let mut acceptor = server.listen().unwrap();
	// Thread pool for accepting
	let pool = TaskPool::new(os::num_cpus());
	// Channel for passing around clients
	let (tx, rx) = channel();
	// Start the client manager
	let manager_tx = tx.clone();
	Thread::spawn(move || manage_clients(manager_tx, rx)).detach();
	// Accept connections
	for request in acceptor.incoming() {
		let client_tx = tx.clone();
		pool.execute(move || process_request(request, client_tx) );
	}
}

fn manage_clients(tx: Sender<WebSocketRemoteClient>, rx: Receiver<WebSocketRemoteClient>) {
	let pool = TaskPool::new(os::num_cpus());
	loop {
		let tx = tx.clone();
		let client = rx.recv();
		pool.execute(move || process_message(client, tx));
	}
}

fn process_message(mut client: WebSocketRemoteClient, tx: Sender<WebSocketRemoteClient>) {
	let received = client.try_recv_message();
	match received {
		Ok(message) => {
			match message {
				// Handle Ping messages by sending Pong messages
				WebSocketMessage::Ping(data) => {
					let message = WebSocketMessage::Pong(data);
					client.send_message(message).ok().expect("Failed to send message");
				}
				// Handle text by echoing it back
				WebSocketMessage::Text(data) => {
					let message = WebSocketMessage::Text(data);
					client.send_message(message).ok().expect("Failed to send message");
				}
				// Handle when the client wants to disconnect
				WebSocketMessage::Close(_) => {
					// Send a close message
					let message = WebSocketMessage::Close(None);
					client.send_message(message).ok().expect("Failed to send message");
					return;
				}
				_ => { }
			}
		}
		Err(err) => match err {
			WebSocketError::NoDataAvailable => { }
			_ => return,
		},
	}
	tx.send(client);
}

fn process_request(request: IoResult<WebSocketInboundRequest>, tx: Sender<WebSocketRemoteClient>) {
	let request = request.ok().expect("Failed to read WebSocket request");
	request.validate().ok().expect("Invalid WebSocket request");
	
	// Let's also check the protocol - if it's not what we want, then fail the connection
	if request.protocol().is_none() || !request.protocol().unwrap().as_slice().contains(&"rust-websocket".to_string()) {
		//let response = request.fail();
		//let _ = response.send_into_inner();
		//return;
	}

	let mut response = request.accept(); // Generate a response
	response.headers.set(WebSocketProtocol(vec!["rust-websocket".to_string()])); // Send a Sec-WebSocket-Protocol header
	let mut client = response.send().ok().expect("Failed to obtain client"); // Send the response
	
	// Send a message
	let message = WebSocketMessage::Text("Hello from the server".to_string());
	client.send_message(message).ok().expect("Failed to send message");
	
	// Consider the connection established, send the client to the manager
	tx.send(client);
}