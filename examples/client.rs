#![allow(unstable)]

extern crate websocket;
extern crate url;

use std::thread::Thread;
use std::io::{Listener, Acceptor};
use std::io::stdin;
use websocket::{WebSocketRequest, WebSocketMessage, Sender, Receiver};
use url::Url;

fn main() {
	let url = Url::parse("ws://127.0.0.1:2794").unwrap();
	let mut request = WebSocketRequest::connect(url).unwrap(); 
	let key = request.key() // Keep this key so we can validate the response
		.unwrap()
		.clone();
	
	let response = request.send().unwrap(); // Send the request and retrieve a response
	response.validate(&key).unwrap(); // Validate the response
	
	// Split the client into a Sender and a Receiver
	let (mut sender, mut receiver) = response.begin().split();
	
	Thread::spawn(move || {
		for message in receiver.incoming_messages() {
			println!("Recv: {:?}", message.unwrap());
		}
	});
	
	loop {
		let input = stdin()
			.read_line()
			.ok()
			.expect("Failed to read line");
		let message = WebSocketMessage::Text(input);
		sender.send_message(message);
	}
}