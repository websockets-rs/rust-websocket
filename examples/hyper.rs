extern crate websocket;
extern crate hyper;

use std::thread;
use std::io::Write;
use websocket::{Server, Message, Sender, Receiver};
use websocket::header::WebSocketProtocol;
use websocket::dataframe::Opcode;
use hyper::Server as HttpServer;
use hyper::server::Handler;
use hyper::net::Fresh;
use hyper::server::request::Request;
use hyper::server::response::Response;

// The HTTP server handler
fn http_handler(_: Request, response: Response<Fresh>) {
	let mut response = response.start().unwrap();
	// Send a client webpage
	response.write_all(b"<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>WebSocket Test</title></head><body><script>var socket=new WebSocket(\"ws://127.0.0.1:2794\", \"rust-websocket\");socket.onmessage=function (event){var received=document.getElementById(\"received\");var br=document.createElement(\"BR\");var text=document.createTextNode(event.data);received.appendChild(br);received.appendChild(text);};function send(element){var input=document.getElementById(element);socket.send(input.value);input.value=\"\";}</script><p id=\"received\"><strong>Received Messages:</strong></p><form onsubmit=\"send('message'); return false\"><input type=\"text\" id=\"message\"><input type=\"submit\" value=\"Send\"></form></body></html>").unwrap();
	response.end().unwrap();
}

fn main() {
	// Start listening for http connections
	thread::spawn(move || {
		let http_server = HttpServer::http("127.0.0.1:8080").unwrap();
		http_server.handle(http_handler).unwrap();
	});

	// Start listening for WebSocket connections
	let ws_server = Server::bind("127.0.0.1:2794").unwrap();

	for connection in ws_server {
		// Spawn a new thread for each connection.
		thread::spawn(move || {
			let request = connection.unwrap().read_request().unwrap(); // Get the request
			let headers = request.headers.clone(); // Keep the headers so we can check them

			request.validate().unwrap(); // Validate the request

			let mut response = request.accept(); // Form a response

			if let Some(&WebSocketProtocol(ref protocols)) = headers.get() {
				if protocols.contains(&("rust-websocket".to_string())) {
					// We have a protocol we want to use
					response.headers.set(WebSocketProtocol(vec!["rust-websocket".to_string()]));
				}
			}

			let mut client = response.send().unwrap(); // Send the response

			let ip = client.get_mut_sender()
				.get_mut()
				.peer_addr()
				.unwrap();

			println!("Connection from {}", ip);

			let message = Message::text("Hello".to_string());
			client.send_message(&message).unwrap();

			let (mut sender, mut receiver) = client.split();

			for message in receiver.incoming_messages() {
				let message: Message = message.unwrap();

				match message.opcode {
					Opcode::Close => {
						let message = Message::close();
						sender.send_message(&message).unwrap();
						println!("Client {} disconnected", ip);
						return;
					},
					Opcode::Ping => {
						let message = Message::pong(message.payload);
						sender.send_message(&message).unwrap();
					},
					_ => sender.send_message(&message).unwrap(),
				}
			}
		});
	}
}
