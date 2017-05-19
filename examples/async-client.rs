extern crate websocket;
extern crate tokio_core;
extern crate tokio_io;
extern crate futures;

const CONNECTION: &'static str = "ws://127.0.0.1:2794";

use std::thread;
use std::io::stdin;

use tokio_core::reactor::Core;

use futures::sink::Sink;
use futures::stream::Stream;
use futures::sync::mpsc;

use websocket::result::WebSocketError;
use websocket::Message;
use websocket::message::Type;
use websocket::client::ClientBuilder;

fn main() {
	println!("Connecting to {}", CONNECTION);

	// standard in isn't supported in mio yet, so we use a thread
	// see https://github.com/carllerche/mio/issues/321
	let (usr_msg, stdin_ch) = mpsc::channel(0);
	thread::spawn(move || {
		let mut input = String::new();
		let mut stdin_sink = usr_msg.wait();
		loop {
			input.clear();
			stdin().read_line(&mut input).unwrap();
			let trimmed = input.trim();

			let (close, msg) = match trimmed {
				"/close" => (true, Message::close()),
				"/ping" => (false, Message::ping(b"PING".to_vec())),
				_ => (false, Message::text(trimmed.to_string())),
			};

			stdin_sink.send(msg)
			          .expect("Sending message across stdin channel.");

			if close {
				break;
			}
		}
	});

	let mut core = Core::new().unwrap();
	let handle = core.handle();

	let client = ClientBuilder::new(CONNECTION)
		.unwrap()
		.add_protocol("rust-websocket")
		.connect_insecure()
		.unwrap()
		.async(&handle)
		.unwrap();

	println!("Successfully connected");

	let (sink, stream) = client.split();

	let runner =
		stream.filter_map(|message| {
			match message.opcode {
				Type::Close => Some(Message::close()),
				Type::Ping => Some(Message::pong(message.payload)),
				_ => {
					// Say what we received
					println!("Received Message: {:?}", message);
					None
				}
			}
		})
		      .select(stdin_ch.map_err(|_| WebSocketError::NoDataAvailable))
		      .forward(sink);

	core.run(runner).unwrap();
}
