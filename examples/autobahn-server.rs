extern crate websocket;

use std::thread;
use std::str::from_utf8;
use websocket::{Server, Message};
use websocket::message::Type;

fn main() {
	let server = Server::bind("127.0.0.1:9002").unwrap();

	for connection in server.filter_map(Result::ok) {

		thread::spawn(move || {
			let client = connection.accept().unwrap();

			let (mut receiver, mut sender) = client.split().unwrap();

			for message in receiver.incoming_messages() {
				let message: Message = match message {
					Ok(message) => message,
					Err(e) => {
						println!("{:?}", e);
						let _ = sender.send_message(&Message::close());
						return;
					}
				};

				match message.opcode {
					Type::Text => {
						let response = Message::text(from_utf8(&*message.payload).unwrap());
						sender.send_message(&response).unwrap()
					}
					Type::Binary => {
						sender.send_message(&Message::binary(message.payload)).unwrap()
					}
					Type::Close => {
						let _ = sender.send_message(&Message::close());
						return;
					}
					Type::Ping => {
						let message = Message::pong(message.payload);
						sender.send_message(&message).unwrap();
					}
					_ => (),
				}
			}
		});
	}
}
