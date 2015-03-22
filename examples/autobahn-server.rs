extern crate websocket;

use std::thread;
use websocket::{Server, Message, Sender, Receiver};

fn main() {
	let addr = "127.0.0.1:9002".to_string();
	
	let server = Server::bind(&addr[..]).unwrap();
	
	for connection in server {
		thread::spawn(move || {
			let request = connection.unwrap().read_request().unwrap();
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
