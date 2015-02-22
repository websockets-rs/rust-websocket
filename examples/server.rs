#![feature(old_io)]

extern crate websocket;

use std::thread;
use std::old_io::{Listener, Acceptor};
use websocket::{Server, Message, Sender, Receiver};

fn main() {
	let server = Server::bind("127.0.0.1:2794").unwrap();

	let mut acceptor = server.listen().unwrap();
	for request in acceptor.incoming() {
		// Spawn a new thread for each connection.
		thread::spawn(move || {
			let request = request.unwrap(); // Get the request
			request.validate().unwrap();
			let response = request.accept(); // Form a response
			let mut client = response.send().unwrap(); // Send the response
			
			let ip = client.get_mut_sender()
				.get_mut()
				.peer_name()
				.unwrap();
			
			println!("Connection from {}", ip);
			
			let message = Message::Text("Hello".to_string());
			client.send_message(message).unwrap();
			
			let (mut sender, mut receiver) = client.split();
			
			for message in receiver.incoming_messages() {
				let message = message.unwrap();
				
				match message {
					Message::Close(_) => {
						let message = Message::Close(None);
						sender.send_message(message).unwrap();
						println!("Client {} disconnected", ip);
						return;
					}
					Message::Ping(data) => {
						let message = Message::Pong(data);
						sender.send_message(message).unwrap();
					}
					_ => sender.send_message(message).unwrap(),
				}
			}
		});
	}
}
