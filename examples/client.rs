#![allow(unstable)]

extern crate websocket;

use std::thread::Thread;
use std::io::stdin;
use websocket::{Message, Sender, Receiver};
use websocket::client::request::Url;
use websocket::Client;

fn main() {
	let url = Url::parse("ws://127.0.0.1:2794").unwrap();
	let request = Client::connect(url).unwrap(); 
	
	let response = request.send().unwrap(); // Send the request and retrieve a response
	response.validate().unwrap(); // Validate the response
	
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
		let message = Message::Text(input);
		sender.send_message(message).unwrap();
	}
}