extern crate hyper;
extern crate websocket;

use hyper::net::Fresh;
use hyper::server::request::Request;
use hyper::server::response::Response;
use hyper::Server as HttpServer;
use std::io::Write;
use std::thread;
use websocket::sync::Server;
use websocket::{Message, OwnedMessage};

const HTML: &'static str = include_str!("websockets.html");

// The HTTP server handler
fn http_handler(_: Request, response: Response<Fresh>) {
	let mut response = response.start().unwrap();
	// Send a client webpage
	response.write_all(HTML.as_bytes()).unwrap();
	response.end().unwrap();
}

fn main() {
	// Start listening for http connections
	thread::spawn(|| {
		let http_server = HttpServer::http("127.0.0.1:8080").unwrap();
		http_server.handle(http_handler).unwrap();
	});

	// Start listening for WebSocket connections
	let ws_server = Server::bind("127.0.0.1:2794").unwrap();

	for connection in ws_server.filter_map(Result::ok) {
		// Spawn a new thread for each connection.
		thread::spawn(|| {
			if !connection
				.protocols()
				.contains(&"rust-websocket".to_string())
			{
				connection.reject().unwrap();
				return;
			}

			let mut client = connection.use_protocol("rust-websocket").accept().unwrap();

			let ip = client.peer_addr().unwrap();

			println!("Connection from {}", ip);

			let message = Message::text("Hello");
			client.send_message(&message).unwrap();

			let (mut receiver, mut sender) = client.split().unwrap();

			for message in receiver.incoming_messages() {
				let message = message.unwrap();

				match message {
					OwnedMessage::Close(_) => {
						let message = Message::close();
						sender.send_message(&message).unwrap();
						println!("Client {} disconnected", ip);
						return;
					}
					OwnedMessage::Ping(data) => {
						let message = Message::pong(data);
						sender.send_message(&message).unwrap();
					}
					_ => sender.send_message(&message).unwrap(),
				}
			}
		});
	}
}
