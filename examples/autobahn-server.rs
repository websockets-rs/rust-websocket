extern crate websocket;

use std::thread::Thread;
use std::os;
use std::old_io::{Listener, Acceptor};
use websocket::{Server, Message, Sender, Receiver};

fn main() {
	let args = os::args();
	let addr = match args.len() {
		1 => "127.0.0.1:9002".to_string(),
		2 => args[1].clone(),
		_ => panic!("Wrong number of arguments"),
	};
	
	let server = Server::bind(&addr[]).unwrap();
	let mut acceptor = server.listen().unwrap();
	
	for request in acceptor.incoming() {
		Thread::spawn(move || {
			let request = request.unwrap();
			request.validate().unwrap();
			let response = request.accept();
			let (mut sender, mut receiver) = response.send().unwrap().split();
			
			for message in receiver.incoming_messages() {
				let message = match message {
					Ok(message) => message,
					Err(e) => {
						println!("{:?}", e);
						let _ = sender.send_message( Message::Close(None));
						return;
					}
				};
				
				match message {
					Message::Text(data) => sender.send_message(Message::Text(data)).unwrap(),
					Message::Binary(data) => sender.send_message(Message::Binary(data)).unwrap(),
					Message::Close(_) => {
						let _ = sender.send_message( Message::Close(None));
						return;
					}
					Message::Ping(data) => {
						let message = Message::Pong(data);
						sender.send_message(message).unwrap();
					}
					_ => (),
				}
			}
		});
	}
}
