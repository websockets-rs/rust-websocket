extern crate websocket;
extern crate futures;
extern crate tokio_core;

use std::thread;
use std::io::stdin;
use tokio_core::reactor::Core;
use futures::future::Future;
use futures::sink::Sink;
use futures::stream::Stream;
use futures::sync::mpsc;
use websocket::result::WebSocketError;
use websocket::{ClientBuilder, OwnedMessage};

const CONNECTION: &'static str = "ws://127.0.0.1:2794";

fn main() {
	println!("Connecting to {}", CONNECTION);
	let mut core = Core::new().unwrap();

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
				"/close" => (true, OwnedMessage::Close(None)),
				"/ping" => (false, OwnedMessage::Ping(b"PING".to_vec())),
				_ => (false, OwnedMessage::Text(trimmed.to_string())),
			};

			stdin_sink.send(msg)
			          .expect("Sending message across stdin channel.");

			if close {
				break;
			}
		}
	});

	let runner = ClientBuilder::new(CONNECTION)
		.unwrap()
		.add_protocol("rust-websocket")
		.async_connect_insecure(&core.handle())
		.and_then(|(duplex, _)| {
			let (sink, stream) = duplex.split();
			stream.filter_map(|message| {
				                  println!("Received Message: {:?}", message);
				                  match message {
				                      OwnedMessage::Close(e) => Some(OwnedMessage::Close(e)),
				                      OwnedMessage::Ping(d) => Some(OwnedMessage::Pong(d)),
				                      _ => None,
				                  }
				                 })
			      .select(stdin_ch.map_err(|_| WebSocketError::NoDataAvailable))
			      .forward(sink)
		});
	core.run(runner).unwrap();
}
