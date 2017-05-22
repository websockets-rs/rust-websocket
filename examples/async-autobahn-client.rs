extern crate websocket;
extern crate tokio_core;
extern crate futures;

use websocket::{ClientBuilder, OwnedMessage};
use websocket::result::WebSocketError;
use tokio_core::reactor::Core;
use futures::sink::Sink;
use futures::stream::Stream;
use futures::Future;
use futures::future::{self, Loop};

fn main() {
	let addr = "ws://127.0.0.1:9001".to_string();
	let agent = "rust-websocket";
	let mut core = Core::new().unwrap();
	let handle = core.handle();

	println!("Using fuzzingserver {}", addr);
	println!("Using agent {}", agent);

	let case_count = get_case_count(addr.clone(), &mut core);
	println!("We will be running {} test cases!", case_count);

	println!("Running test suite...");
	for case_id in 1..(case_count + 1) {
		let url = addr.clone() + "/runCase?case=" + &case_id.to_string()[..] + "&agent=" + agent;

		let test_case = ClientBuilder::new(&url)
			.unwrap()
			.async_connect_insecure(&handle)
			.and_then(move |(duplex, _)| {
				println!("Executing test case: {}/{}", case_id, case_count);
				future::loop_fn(duplex, |stream| {
					stream.into_future()
					      .or_else(|(err, stream)| {
						               println!("Could not receive message: {:?}", err);
						               stream.send(OwnedMessage::Close(None)).map(|s| (None, s))
						              })
					      .and_then(|(msg, stream)| match msg {
					                    Some(OwnedMessage::Text(txt)) => {
						                    stream.send(OwnedMessage::Text(txt))
					                              .map(|s| Loop::Continue(s))
					                              .boxed()
					                    }
					                    Some(OwnedMessage::Binary(bin)) => {
						                    stream.send(OwnedMessage::Binary(bin))
					                              .map(|s| Loop::Continue(s))
					                              .boxed()
					                    }
					                    Some(OwnedMessage::Ping(data)) => {
						                    stream.send(OwnedMessage::Pong(data))
					                              .map(|s| Loop::Continue(s))
					                              .boxed()
					                    }
					                    Some(OwnedMessage::Close(_)) => {
						                    stream.send(OwnedMessage::Close(None))
					                              .map(|_| Loop::Break(()))
					                              .boxed()
					                    }
					                    Some(OwnedMessage::Pong(_)) => {
						                    future::ok(Loop::Continue(stream)).boxed()
					                    }
					                    None => future::ok(Loop::Break(())).boxed(),
					                })
				})
			})
			.map(move |_| {
				     println!("Test case {} is finished!", case_id);
				    })
			.or_else(move |err| {
				         println!("Test case {} ended with an error: {:?}", case_id, err);
				         Ok(()) as Result<(), ()>
				        });

		core.run(test_case).ok();
	}

	update_reports(addr.clone(), agent, &mut core);
	println!("Test suite finished!");
}

fn get_case_count(addr: String, core: &mut Core) -> usize {
	let url = addr + "/getCaseCount";
	let err = "Unsupported message in /getCaseCount";

	let counter = ClientBuilder::new(&url)
		.unwrap()
		.async_connect_insecure(&core.handle())
		.and_then(|(s, _)| s.into_future().map_err(|e| e.0))
		.and_then(|(msg, _)| match msg {
		              Some(OwnedMessage::Text(txt)) => Ok(txt.parse().unwrap()),
		              _ => Err(WebSocketError::ProtocolError(err)),
		          });
	core.run(counter).unwrap()
}

fn update_reports(addr: String, agent: &str, core: &mut Core) {
	println!("Updating reports...");
	let url = addr + "/updateReports?agent=" + agent;

	let updater = ClientBuilder::new(&url)
		.unwrap()
		.async_connect_insecure(&core.handle())
		.and_then(|(sink, _)| sink.send(OwnedMessage::Close(None)));
	core.run(updater).unwrap();

	println!("Reports updated.");
}
