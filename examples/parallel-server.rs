extern crate futures;
extern crate tokio;
extern crate websocket;

use websocket::message::OwnedMessage;
use websocket::r#async::Server;
use websocket::server::InvalidConnection;

use tokio::runtime;
use tokio::runtime::TaskExecutor;

use futures::future::{self, Loop};
use futures::sync::mpsc;
use futures::{Future, Sink, Stream};

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

type Id = u32;

fn main() {
	let runtime = runtime::Builder::new().build().unwrap();
	let executor = runtime.executor();
	let server = Server::bind("127.0.0.1:8081", &tokio::reactor::Handle::default())
		.expect("Failed to create server");
	let connections = Arc::new(RwLock::new(HashMap::new()));
	let (receive_channel_out, receive_channel_in) = mpsc::unbounded();
	let conn_id = Arc::new(RwLock::new(Counter::new()));
	let connections_inner = connections.clone();
	// Handle new connection
	let connection_handler = server
		.incoming()
		// we don't wanna save the stream if it drops
		.map_err(|InvalidConnection { error, .. }| error)
		.for_each(move |(upgrade, addr)| {
			let connections_inner = connections_inner.clone();
			println!("Got a connection from: {}", addr);
			let channel = receive_channel_out.clone();
			let executor_inner = executor.clone();
			let conn_id = conn_id.clone();
			let f = upgrade.accept().and_then(move |(framed, _)| {
				let id = conn_id
					.write()
					.unwrap()
					.next()
					.expect("maximum amount of ids reached");
				let (sink, stream) = framed.split();
				let f = channel.send((id, stream));
				spawn_future(f, "Send stream to connection pool", &executor_inner);
				connections_inner.write().unwrap().insert(id, sink);
				Ok(())
			});
			spawn_future(f, "Handle new connection", &executor);
			Ok(())
		})
		.map_err(|_| ());

	// Handle receiving messages from a client
	let receive_handler = receive_channel_in.for_each(|(id, stream)| {
		stream
			.for_each(move |msg| {
				process_message(id, &msg);
				Ok(())
			})
			.map_err(|_| ())
	});

	let (send_channel_out, send_channel_in) = mpsc::unbounded();

	// Handle sending messages to a client
	let connections_inner = connections.clone();
	let executor = runtime.executor();
	let executor_inner = executor.clone();
	let send_handler = send_channel_in
		.for_each(move |(id, msg): (Id, String)| {
			let connections = connections_inner.clone();
			let sink = connections
				.write()
				.unwrap()
				.remove(&id)
				.expect("Tried to send to invalid client id");

			println!("Sending message '{}' to id {}", msg, id);
			let f = sink
				.send(OwnedMessage::Text(msg))
				.and_then(move |sink| {
					connections.write().unwrap().insert(id, sink);
					Ok(())
				})
				.map_err(|_| ());
			executor_inner.spawn(f);
			Ok(())
		})
		.map_err(|_| ());

	// Main 'logic' loop
	let main_loop = future::loop_fn((), move |_| {
		let connections = connections.clone();
		let send_channel_out = send_channel_out.clone();
		let executor = executor.clone();
		tokio::timer::Delay::new(Instant::now() + Duration::from_millis(100))
			.map_err(|_| ())
			.and_then(move |_| {
				let should_continue = update(connections, send_channel_out, &executor);
				match should_continue {
					Ok(true) => Ok(Loop::Continue(())),
					Ok(false) => Ok(Loop::Break(())),
					Err(()) => Err(()),
				}
			})
	});

	let handlers =
		main_loop.select2(connection_handler.select2(receive_handler.select(send_handler)));

	runtime
		.block_on_all(handlers)
		.map_err(|_| println!("Error while running core loop"))
		.unwrap();
}

fn spawn_future<F, I, E>(f: F, desc: &'static str, executor: &TaskExecutor)
where
	F: Future<Item = I, Error = E> + 'static + Send,
	E: Debug,
{
	executor.spawn(
		f.map_err(move |e| println!("Error in {}: '{:?}'", desc, e))
			.map(move |_| println!("{}: Finished.", desc)),
	);
}

fn process_message(id: u32, msg: &OwnedMessage) {
	if let OwnedMessage::Text(ref txt) = *msg {
		println!("Received message '{}' from id {}", txt, id);
	}
}

type SinkContent = websocket::client::r#async::Framed<
	tokio::net::TcpStream,
	websocket::r#async::MessageCodec<OwnedMessage>,
>;
type SplitSink = futures::stream::SplitSink<SinkContent>;
// Represents one tick in the main loop
fn update(
	connections: Arc<RwLock<HashMap<Id, SplitSink>>>,
	channel: mpsc::UnboundedSender<(Id, String)>,
	executor: &TaskExecutor,
) -> Result<bool, ()> {
	let executor_inner = executor.clone();
	executor.spawn(futures::lazy(move || {
		for (id, _) in connections.read().unwrap().iter() {
			let f = channel.clone().send((*id, "Hi there!".to_owned()));
			spawn_future(f, "Send message to write handler", &executor_inner);
		}
		Ok(())
	}));
	Ok(true)
}

struct Counter {
	count: Id,
}
impl Counter {
	fn new() -> Self {
		Counter { count: 0 }
	}
}

impl Iterator for Counter {
	type Item = Id;

	fn next(&mut self) -> Option<Id> {
		if self.count != Id::max_value() {
			self.count += 1;
			Some(self.count)
		} else {
			None
		}
	}
}
