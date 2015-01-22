#![allow(unstable)]

extern crate websocket;

use std::thread::Thread;
use std::io::{Listener, Acceptor};
use websocket::{WebSocketServer, WebSocketMessage, WebSocketStream, Sender, Receiver};

fn main() {
	let server = WebSocketServer::bind("127.0.0.1:2794").unwrap();

	let mut acceptor = server.listen().unwrap();
	for request in acceptor.incoming() {
		// Spawn a new thread for each connection.
		Thread::spawn(move || {
			let request = request.unwrap(); // Get the request
			request.validate().unwrap();
			let response = request.accept(); // Form a response
			let mut client = response.send().unwrap(); // Send the response
			
			let name = match *client.get_mut_sender().get_mut() {
				WebSocketStream::Tcp(ref mut inner) => inner.peer_name().unwrap(),
				WebSocketStream::Ssl(ref mut inner) => inner.get_mut().peer_name().unwrap(),
			};
			
			println!("Connection from {}", name);
			
			let message = WebSocketMessage::Text("Hello".to_string());
			client.send_message(message).unwrap();
			
			let (mut sender, mut receiver) = client.split();
			
			for message in receiver.incoming_messages() {
				let message = message.unwrap();
				
				match message {
					WebSocketMessage::Close(_) => {
						let message = WebSocketMessage::Close(None);
						sender.send_message(message).unwrap();
						println!("Client {} disconnected", name);
						return;
					}
					WebSocketMessage::Ping(data) => {
						let message = WebSocketMessage::Pong(data);
						sender.send_message(message).unwrap();
					}
					_ => sender.send_message(message).unwrap(),
				}
			}
		});
	}
}