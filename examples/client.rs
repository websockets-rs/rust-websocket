extern crate websocket;

fn main() {
	use std::thread;
	use std::sync::mpsc::sync_channel;
	use std::io::stdin;

	use websocket::{Message, Sender, Receiver};
    use websocket::message::Type;
	use websocket::client::request::Url;
	use websocket::Client;

	let url = Url::parse("ws://127.0.0.1:2794").unwrap();

	println!("Connecting to {}", url);

	let request = Client::connect(url).unwrap();

	let response = request.send().unwrap(); // Send the request and retrieve a response

	println!("Validating response...");

	response.validate().unwrap(); // Validate the response

	println!("Successfully connected");

	let (mut sender, mut receiver) = response.begin().split();

	let (tx, rx) = sync_channel(10);

	let tx_1 = tx.clone();

	let send_loop = thread::spawn(move || {
		loop {
			// Send loop
			let message: Message = match rx.recv() {
				Ok(m) => m,
				Err(e) => {
					println!("Send Loop: {:?}", e);
					return;
				}
			};
			match message.opcode {
				Type::Close => {
					let _ = sender.send_message(&message);
					// If it's a close message, just send it and then return.
					return;
				},
				_ => (),
			}
			// Send the message
			match sender.send_message(&message) {
				Ok(()) => (),
				Err(e) => {
					println!("Send Loop: {:?}", e);
					let _ = sender.send_message(&Message::close());
					return;
				}
			}
		}
	});

	let receive_loop = thread::spawn(move || {
		// Receive loop
		for message in receiver.incoming_messages() {
			let message: Message = match message {
				Ok(m) => m,
				Err(e) => {
					println!("Receive Loop: {:?}", e);
					let _ = tx_1.send(Message::close());
					return;
				}
			};
			match message.opcode {
				Type::Close => {
					// Got a close message, so send a close message and return
					let _ = tx_1.send(Message::close());
					return;
				}
				Type::Ping => match tx_1.send(Message::pong(message.payload)) {
					// Send a pong in response
					Ok(()) => (),
					Err(e) => {
						println!("Receive Loop: {:?}", e);
						return;
					}
				},
				// Say what we received
				_ => println!("Receive Loop: {:?}", message),
			}
		}
	});

	loop {
		let mut input = String::new();

		stdin().read_line(&mut input).unwrap();

		let trimmed = input.trim();

		let message = match trimmed {
			"/close" => {
				// Close the connection
				let _ = tx.send(Message::close());
				break;
			}
			// Send a ping
			"/ping" => Message::ping(b"PING".to_vec()),
			// Otherwise, just send text
			_ => Message::text(trimmed.to_string()),
		};

		match tx.send(message) {
			Ok(()) => (),
			Err(e) => {
				println!("Main Loop: {:?}", e);
				break;
			}
		}
	}

	// We're exiting

	println!("Waiting for child threads to exit");

	let _ = send_loop.join();
	let _ = receive_loop.join();

	println!("Exited");
}
