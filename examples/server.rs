extern crate websocket;

use std::thread;
use websocket::{Server, Message};
use websocket::message::Type;
use websocket::header::WebSocketProtocol;

// TODO: I think the .reject() call is only for malformed packets
// there should be an easy way to accept the socket with the given protocols
// this would mean there should be a way to accept or reject on the client
// Do you send the protocol you want to talk when you are not given it as an
// option? What is a rejection response? Does the client check for it?
// Client should expose what the decided protocols/extensions/etc are.
// can you accept only one protocol??

fn main() {
	  let server = Server::bind("127.0.0.1:2794").unwrap();

	  for request in server {
		    // Spawn a new thread for each connection.
		    thread::spawn(move || {
            if !request.protocols().contains(&"rust-websocket".to_string()) {
                request.reject().unwrap();
                return;
            }

			      let mut client = request.accept().unwrap();

			      let ip = client.peer_addr().unwrap();

			      println!("Connection from {}", ip);

			      let message: Message = Message::text("Hello".to_string());
			      client.send_message(&message).unwrap();

			      let (mut receiver, mut sender) = client.split().unwrap();

			      for message in receiver.incoming_messages() {
				        let message: Message = message.unwrap();

				        match message.opcode {
					          Type::Close => {
						            let message = Message::close();
						            sender.send_message(&message).unwrap();
						            println!("Client {} disconnected", ip);
						            return;
					          },
					          Type::Ping => {
						            let message = Message::pong(message.payload);
						            sender.send_message(&message).unwrap();
					          }
					          _ => sender.send_message(&message).unwrap(),
				        }
			      }
		    });
	  }
}
