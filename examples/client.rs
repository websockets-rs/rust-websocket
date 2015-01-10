extern crate websocket;
extern crate url;

use url::Url;
use websocket::handshake::WebSocketRequest;
use websocket::message::WebSocketMessage;
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
	
	// Send a fragmented message
	{
		let mut fragment_sender = client.frag_send_text("This ").unwrap();
		let _ = fragment_sender.send("is ");
		let _ = fragment_sender.send("a ");
		let _ = fragment_sender.send("fragmented ");
		let _ = fragment_sender.finish("message.");
	}
	
	let message = WebSocketMessage::Text("Hello from the client".to_string());
	let _ = client.send_message(message);
	
	for message in client.incoming_messages() {
		println!("Recv: {:?}", message.unwrap());
	}
}