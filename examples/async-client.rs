extern crate futures;
extern crate tokio;
extern crate websocket;

use futures::future::Future;
use futures::sink::Sink;
use futures::stream::Stream;
use futures::sync::mpsc;
use std::io::stdin;
use std::thread;
use websocket::result::WebSocketError;
use websocket::{ClientBuilder, OwnedMessage};

const CONNECTION: &'static str = "ws://127.0.0.1:2794";

// Async websocket chat client
fn main() {
	println!("Connecting to {}", CONNECTION);

	// Construct new Tokio runtime environment
	let mut runtime = tokio::runtime::current_thread::Builder::new()
		.build()
		.unwrap();

	let (usr_msg, stdin_ch) = mpsc::channel(0);

	// Spawn new thread to read user input
	thread::spawn(|| {
		let mut input = String::new();
		let mut stdin_sink = usr_msg.wait();
		loop {
			// Read user input from stdin
			input.clear();
			stdin().read_line(&mut input).unwrap();

			// Trim whitespace and match input to known chat commands
			// If input is unknown, send trimmed input as a chat message
			let trimmed = input.trim();
			let (close, msg) = match trimmed {
				"/close" => (true, OwnedMessage::Close(None)),
				"/ping" => (false, OwnedMessage::Ping(b"PING".to_vec())),
				_ => (false, OwnedMessage::Text(trimmed.to_string())),
			};
			// Send message to websocket server
			stdin_sink
				.send(msg)
				.expect("Sending message across stdin channel.");
			// If user entered the "/close" command, break the loop
			if close {
				break;
			}
		}
	});

	// Construct a new connection to the websocket server
	let runner = ClientBuilder::new(CONNECTION)
		.unwrap()
		.add_protocol("rust-websocket")
		.async_connect_insecure()
		.and_then(|(duplex, _)| {
			let (sink, stream) = duplex.split();
			stream
				// Iterate over message as they arrive in stream
				.filter_map(|message| {
					println!("Received Message: {:?}", message);
					// Respond to close or ping commands from the server
					match message {
						OwnedMessage::Ping(d) => Some(OwnedMessage::Pong(d)),
						_ => None,
					}
				})
				// Takes in messages from both sinks
				.select(stdin_ch.map_err(|_| WebSocketError::NoDataAvailable))
				// Return a future that completes once all incoming data from the above streams has been processed into the sink
				.forward(sink)
		});
	// Start our websocket client runner in the Tokio environment
	let _ = runtime.block_on(runner).unwrap();
}
