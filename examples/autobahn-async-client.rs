extern crate websocket;
extern crate rustc_serialize as serialize;
extern crate tokio_core;
extern crate tokio_io;
extern crate futures;

use websocket::ClientBuilder;
use websocket::Message;
use websocket::message::Type;
use websocket::result::WebSocketError;
use websocket::buffer::BufReader;
use serialize::json;

use tokio_core::net::TcpStream;
use tokio_core::reactor::Core;
use tokio_core::reactor::Handle;
use tokio_io::AsyncRead;
use tokio_io::codec::Framed;
use futures::sink::Sink;
use futures::stream::Stream;
use futures::Future;
use futures::future;
use futures::future::Loop;
use websocket::codec::MessageCodec;
use websocket::codec::Context;

fn connect_ws<'m>(
	url: &str,
	handle: &Handle,
) -> Framed<BufReader<TcpStream>, MessageCodec<'m, Message<'m>>> {
	let client = ClientBuilder::new(url)
		.unwrap()
		.connect_insecure()
		.unwrap();

	let (stream, buf_data) = client.into_stream();

	let stream = TcpStream::from_stream(stream, handle).unwrap();

	let buf_stream = match buf_data {
		Some((buf, pos, cap)) => BufReader::from_parts(stream, buf, pos, cap),
		None => BufReader::new(stream),
	};

	buf_stream.framed(<MessageCodec<Message>>::default(Context::Client))
}

fn main() {
	let addr = "ws://127.0.0.1:9001".to_string();
	let agent = "rust-websocket";
	let mut core = Core::new().unwrap();
	let handle = core.handle();

	println!("Using fuzzingserver {}", addr);
	println!("Using agent {}", agent);
	let case_count = get_case_count(addr.clone(), &handle);

	println!("Running test suite...");
	for case_id in 1..case_count {
		let url = addr.clone() + "/runCase?case=" + &case_id.to_string()[..] + "&agent=" + agent;
		let handle = core.handle();

		let duplex = connect_ws(&url, &handle);

		println!("Executing test case: {}/{}", case_id, case_count);

		core.run(future::loop_fn(duplex, |stream| {
			stream.into_future()
			      .or_else(|(err, stream)| {
				               println!("message transmission was an error: {:?}", err);
				               stream.send(Message::close()).map(|s| (None, s))
				              })
			      .and_then(|(msg, stream)| match msg.map(|m| (m.opcode, m)) {
			                    Some((Type::Text, m)) => {
				                    if let Ok(txt) = String::from_utf8(m.payload.into_owned()) {
				                        stream.send(Message::text(txt))
				                              .map(|s| Loop::Continue(s))
				                              .boxed()
				                       } else {
				                        stream.send(Message::close())
				                              .map(|_| Loop::Break(()))
				                              .boxed()
				                       }
			                    }
			                    Some((Type::Binary, m)) => {
				println!("payload length: {}", m.payload.len());
				stream.send(Message::binary(m.payload.into_owned()))
				      .map(|s| Loop::Continue(s))
				      .boxed()
			}
			                    Some((Type::Ping, m)) => {
				                    stream.send(Message::pong(m.payload))
			                              .map(|s| Loop::Continue(s))
			                              .boxed()
			                    }
			                    Some((Type::Close, _)) => {
				                    stream.send(Message::close())
			                              .map(|_| Loop::Break(()))
			                              .boxed()
			                    }
			                    Some((Type::Pong, _)) => future::ok(Loop::Continue(stream)).boxed(),
			                    None => future::ok(Loop::Break(())).boxed(),
			                })
			      .map_err(|e| {
				               println!("Error sending or other: {:?}", e);
				               e
				              })
		}))
		    .unwrap();
	}

	update_reports(addr.clone(), agent, &handle);
}

fn get_case_count(addr: String, handle: &Handle) -> usize {
	let url = addr + "/getCaseCount";

	connect_ws(&url, &handle)
		.into_future()
		.map_err(|e| e.0)
		.and_then(|(msg, _)| match msg.map(|m| (m.opcode, m)) {
		              Some((Type::Text, message)) => {
			let count = String::from_utf8(message.payload.into_owned()).unwrap();
			let count: usize = json::decode(&count).unwrap();
			println!("We will be running {} test cases!", count);
			Ok(count)
		}
		              _ => {
			let error = WebSocketError::ProtocolError("Unsupported message in /getCaseCount");
			Err(error)
		}
		          })
		.wait()
		.unwrap()
}

fn update_reports(addr: String, agent: &str, handle: &Handle) {
	let url = addr + "/updateReports?agent=" + agent;
	let (sink, stream) = connect_ws(&url, &handle).split();

	println!("Updating reports...");

	stream.filter(|m| m.opcode == Type::Close)
	      .take(1)
	      .map(|_| Message::close())
	      .forward(sink)
	      .wait()
	      .unwrap();

	println!("Reports updated.");
	println!("Test suite finished!");
}
