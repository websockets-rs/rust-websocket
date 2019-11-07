extern crate futures;
extern crate tokio;
extern crate websocket;

use futures::future::{self, Loop};
use futures::sink::Sink;
use futures::stream::Stream;
use futures::Future;
use tokio::runtime::current_thread::Runtime;
use websocket::result::WebSocketError;
use websocket::{ClientBuilder, OwnedMessage};

fn main() {
	let addr = "ws://127.0.0.1:9001".to_string();
	let agent = "rust-websocket";
	let mut runtime = tokio::runtime::current_thread::Builder::new()
		.build()
		.unwrap();

	println!("Using fuzzingserver {}", addr);
	println!("Using agent {}", agent);

	let case_count = get_case_count(addr.clone(), &mut runtime);
	println!("We will be running {} test cases!", case_count);

	println!("Running test suite...");
	for case_id in 1..(case_count + 1) {
		let url = addr.clone() + "/runCase?case=" + &case_id.to_string()[..] + "&agent=" + agent;

		let test_case = ClientBuilder::new(&url)
			.unwrap()
			.async_connect_insecure()
			.and_then(|(duplex, _)| {
				println!("Executing test case: {}/{}", case_id, case_count);
				future::loop_fn(duplex, |stream| {
					stream
						.into_future()
						.or_else(|(err, stream)| {
							println!("Could not receive message: {:?}", err);
							stream.send(OwnedMessage::Close(None)).map(|s| (None, s))
						})
						.and_then(|(msg, stream)| -> Box<dyn Future<Item = _, Error = _>> {
							match msg {
								Some(OwnedMessage::Text(txt)) => Box::new(
									stream
										.send(OwnedMessage::Text(txt))
										.map(|s| Loop::Continue(s)),
								),
								Some(OwnedMessage::Binary(bin)) => Box::new(
									stream
										.send(OwnedMessage::Binary(bin))
										.map(|s| Loop::Continue(s)),
								),
								Some(OwnedMessage::Ping(data)) => Box::new(
									stream
										.send(OwnedMessage::Pong(data))
										.map(|s| Loop::Continue(s)),
								),
								Some(OwnedMessage::Close(_)) => Box::new(
									stream
										.send(OwnedMessage::Close(None))
										.map(|_| Loop::Break(())),
								),
								Some(OwnedMessage::Pong(_)) => {
									Box::new(future::ok(Loop::Continue(stream)))
								}
								None => Box::new(future::ok(Loop::Break(()))),
							}
						})
				})
			})
			.map(|_| {
				println!("Test case {} is finished!", case_id);
			})
			.or_else(|err| {
				println!("Test case {} ended with an error: {:?}", case_id, err);
				Ok(()) as Result<(), ()>
			});

		runtime.block_on(test_case).ok();
	}

	update_reports(addr.clone(), agent, &mut runtime);
	println!("Test suite finished!");
}

fn get_case_count(addr: String, runtime: &mut Runtime) -> usize {
	let url = addr + "/getCaseCount";
	let err = "Unsupported message in /getCaseCount";

	let counter = ClientBuilder::new(&url)
		.unwrap()
		.async_connect_insecure()
		.and_then(|(s, _)| s.into_future().map_err(|e| e.0))
		.and_then(|(msg, _)| match msg {
			Some(OwnedMessage::Text(txt)) => Ok(txt.parse().unwrap()),
			_ => Err(WebSocketError::ProtocolError(err)),
		});
	runtime.block_on(counter).unwrap()
}

fn update_reports(addr: String, agent: &str, runtime: &mut Runtime) {
	println!("Updating reports...");
	let url = addr + "/updateReports?agent=" + agent;

	let updater = ClientBuilder::new(&url)
		.unwrap()
		.async_connect_insecure()
		.and_then(|(sink, _)| sink.send(OwnedMessage::Close(None)));
	runtime.block_on(updater).unwrap();

	println!("Reports updated.");
}
