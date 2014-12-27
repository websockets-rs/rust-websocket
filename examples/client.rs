extern crate websocket;
extern crate url;

use url::Url;
use std::thread::Thread;
use websocket::handshake::WebSocketRequest;
use websocket::message::WebSocketMessage;
use websocket::client::fragment::string_fragmenter;
use websocket::header::WebSocketProtocol;

fn main() {
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
	
	let message = WebSocketMessage::Text("Hello from the client".to_string());
	let _ = client.send_message(message);
	
	let (mut writer, iterator) = string_fragmenter();
	Thread::spawn(move || {			
		writer.push("This ");
		writer.push("is ");
		writer.push("a ");
		writer.push("fragmented ");
		writer.push("message.");
		writer.finish();			
	}).detach();
	
	let _ = client.frag_send_text(iterator);
	
	for message in client.incoming_messages() {
		println!("Recv: {}", message.unwrap());
	}
}