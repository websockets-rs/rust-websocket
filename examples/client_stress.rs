extern crate websocket;
extern crate url;

use std::thread;
use std::io::timer::sleep;
use std::time::duration::Duration;
use url::Url;
use websocket::handshake::WebSocketRequest;
use websocket::message::WebSocketMessage;
use websocket::header::WebSocketProtocol;

fn main() {
	for i in range (1us, 11) {
		for _ in range(0us, 1000) {
			thread::Builder::new().stack_size(128 * 1024).spawn(move || {
				let url = Url::parse("ws://127.0.0.1:2794").unwrap();
				let mut request = WebSocketRequest::connect(url).unwrap(); 
				let key = request.key().unwrap().clone(); // Keep this key so we can validate the response
				
				// Let's also set a Sec-WebSocket-Protocol
				let protocol = WebSocketProtocol(vec!["rust-websocket".to_string()]);
				request.headers.set(protocol);
				
				// Retrieve the response from the server
				let response = request.send().unwrap();
				response.validate(&key).unwrap(); // Ensure the response is valid, and panic if it isn't
				
				let mut client = response.begin();

				let message = WebSocketMessage::Text("Hello".to_string());
				let _ = client.send_message(message);
				
				let mut client_cloned = client.clone();
				for _ in client.incoming_messages() {
					let message = WebSocketMessage::Text("Hello".to_string());
					let _ = client_cloned.send_message(message);
					sleep(Duration::milliseconds(1000));
				}
			});
			sleep(Duration::milliseconds(1));
		}
		println!("Connection {:?}000", i);
	}
	loop {}
}