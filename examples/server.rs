extern crate websocket;
//extern crate openssl;

use std::thread::Thread;
use std::io::{Listener, Acceptor};
use websocket::{WebSocketServer, WebSocketMessage};
//use openssl::ssl::{SslContext, SslMethod};
//use openssl::x509::X509FileType;

fn main() {
	/*
	let mut context = SslContext::new(SslMethod::Tlsv1).unwrap();
	let _ = context.set_certificate_file(&(Path::new("cert.pem")), X509FileType::PEM);
	let _ = context.set_private_key_file(&(Path::new("key.pem")), X509FileType::PEM);
	
	let server = WebSocketServer::bind_secure("127.0.0.1:2794", &context).unwrap();
	*/
	let server = WebSocketServer::bind("127.0.0.1:2794").unwrap();
	
	let mut acceptor = server.listen().unwrap();

	let mut id = 0u;
	
	for request in acceptor.incoming() {
		id += 1;
		Thread::spawn(move || {
			println!("Connection [{}]", id);
			let request = request.unwrap();
			
			// Let's also check the protocol - if it's not what we want, then fail the connection
			if request.protocol().is_none() || !request.protocol().unwrap().as_slice().contains(&"rust-websocket".to_string()) {
				let response = request.fail();
				let _ = response.send_into_inner();
				return;
			}
			
			let response = request.accept(); // Generate a response
			let mut client = response.send().unwrap(); // Send the response
			
			// Send a message
			let message = WebSocketMessage::Text("Hello from the server".to_string());
			let _ = client.send_message(message);
			
			// Send a fragmented message
			{
				let mut fragment_sender = client.frag_send_text("This ").unwrap();
				let _ = fragment_sender.send("is ");
				let _ = fragment_sender.send("a ");
				let _ = fragment_sender.send("fragmented ");
				let _ = fragment_sender.finish("message.");
			}
			
			let mut client_captured = client.clone();
			
			for message in client.incoming_messages() {
				match message {
					Ok(message) => {
						println!("Recv [{}]: {}", id, message);
						
						match message {
							// Handle Ping messages by sending Pong messages
							WebSocketMessage::Pong(data) => {
								let message = WebSocketMessage::Pong(data);
								let _ = client_captured.send_message(message);
								println!("Closed connection {}", id);
								// Close the connection
								break;
							}
							// Handle when the client wants to disconnect
							WebSocketMessage::Close(_) => {
								// Send a close message
								let message = WebSocketMessage::Close(None);
								let _ = client_captured.send_message(message);
								println!("Closed connection {}", id);
								// Close the connection
								break;
							}
							_ => { }
						}
						
						let message = WebSocketMessage::Text("Response from the server".to_string());
						let _ = client_captured.send_message(message);
					}
					Err(err) => {
						println!("Error [{}]: {}",id, err);
						break;
					}
				}
			}
		}).detach();
	}
}