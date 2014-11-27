Rust-WebSocket
==============

Rust-WebSocket is a WebSocket ([RFC6455](http://datatracker.ietf.org/doc/rfc6455/)) library written in Rust.

Rust-WebSocket attempts to provide a framework for WebSocket connections (both clients and servers). The library is currently in an experimental state, but can work as a simple WebSocket server or client, with more functionality to come. There is some documentation, but no testing code at the moment, however that is being worked on.

## Installation

To add the library to a Cargo project, add this to your Cargo.toml:
```INI
[dependencies.websocket]

git = "https://github.com/cyderize/rust-websocket.git"
```

And add ```extern crate websocket;``` to your project.

## Usage

Simple WebSocket server that can send messages:
```Rust
extern crate websocket;

use websocket::{WebSocketServer, WebSocketClient};
use websocket::message::WebSocketMessage;
use websocket::handshake::WebSocketResponse;
use std::io::{Listener, Acceptor};
use std::io::stdin;

fn handle_client(mut client: WebSocketClient) {
	//Get the request (but don't move 'client')
	let request = client.request.clone().unwrap();
	let key = request.headers.get("Sec-WebSocket-Key").unwrap();
	
	//Form a response from the key
	let response = WebSocketResponse::new(key.as_slice(), None);
	//Send the response to the client
	client.send_handshake_response(response);
	
	//Now we can send and receive messages
	let mut tx = client.sender();
	let rx = client.receiver();

	spawn(proc() {
		//Since this blocks, we'll put it in another task
		for message in rx.incoming() {
			match message.unwrap() {
				WebSocketMessage::Text(message) => {
					println!("Message received: {}", message);
				}
				_ => { /* Non-text message received */ }
			}
		}
	});
	
	loop {
		println!("Send a message:");
		let input = stdin().read_line().ok().expect("Failed to read line");
		let message = WebSocketMessage::Text(input);
		
		tx.send_message(&message);
	}
}

fn main() {
	let server = WebSocketServer::bind("127.0.0.1:1234").unwrap();
	let mut acceptor = server.listen();
	
	for client in acceptor.incoming() {
		match client {
			Ok(client) => {
				spawn(proc() {
					handle_client(client)
				})
			}
			Err(e) => { /* An error occurred */ }
		}
	}
}
```

And to go with it, a Rust-WebSocket based client:
```Rust
extern crate websocket;

use websocket::WebSocketClient;
use websocket::message::WebSocketMessage;
use websocket::handshake::WebSocketRequest;
use std::io::stdin;

fn main() {
	let request = WebSocketRequest::new("ws://127.0.0.1:1234", "myProtocol");
	let client = WebSocketClient::connect(request);
	
	match client {
		Ok(client) => {
			let rx = client.receiver();
			let mut tx = client.sender();
			
			spawn(proc() {
				//Since this blocks, we'll put it in another task
				for message in rx.incoming() {
					match message.unwrap() {
						WebSocketMessage::Text(message) => {
							println!("Message received: {}", message);
						}
						_ => { /* Non text message */ }
					}
				}
			});
			
			loop {
				println!("Send a message:");
				let input = stdin().read_line().ok().expect("Failed to read line");
				let message = WebSocketMessage::Text(input);
				
				tx.send_message(&message);
			}
		}
		Err(e) => { println!("Cannot connect: {}", e); }
	}
}
```

Or a browser-based client:
```HTML
<!DOCTYPE html>
<html>
	<head>
		<meta charset="utf-8">
		<title>Websockets</title>
	</head>
	<body>
		<script>
			var socket = new WebSocket("ws://127.0.0.1:1234", "protocolOne");
			socket.onmessage = function (event) {
				document.getElementById("response").innerHTML += event.data + "<br>";
			}
			
			function send() {
				var messagebox = document.getElementById("message");
				var message = messagebox.value;
				socket.send(message); 
				messagebox.value = "";		
			}
		</script>
		<div id="response">
		</div>
		<form action="#" onsubmit="send()">
			<input type="text" id="message">
			<input type="submit" value="Send">
		</form>
	</body>
</html>
```

## License

### The MIT License (MIT)

Copyright (c) 2014 Cyderize

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
