extern crate futures;
extern crate tokio;
extern crate websocket;

use websocket::message::OwnedMessage;
use websocket::r#async::Server;
use websocket::server::InvalidConnection;

use futures::{Future, Sink, Stream};

fn main() {
	let mut runtime = tokio::runtime::Builder::new().build().unwrap();
	let executor = runtime.executor();
	// bind to the server
	let server = Server::bind("127.0.0.1:9002", &tokio::reactor::Handle::default()).unwrap();

	// time to build the server's future
	// this will be a struct containing everything the server is going to do

	// a stream of incoming connections
	let f = server
		.incoming()
		// we don't wanna save the stream if it drops
		.map_err(|InvalidConnection { error, .. }| error)
		.for_each(move |(upgrade, addr)| {
			// accept the request to be a ws connection
			println!("Got a connection from: {}", addr);
			let f = upgrade.accept().and_then(|(s, _)| {
				// simple echo server impl
				let (sink, stream) = s.split();
				stream
					.take_while(|m| Ok(!m.is_close()))
					.filter_map(|m| match m {
						OwnedMessage::Ping(p) => Some(OwnedMessage::Pong(p)),
						OwnedMessage::Pong(_) => None,
						_ => Some(m),
					})
					.forward(sink)
					.and_then(|(_, sink)| sink.send(OwnedMessage::Close(None)))
			});

			executor.spawn(
				f.map_err(move |e| println!("{}: '{:?}'", addr, e))
					.map(move |_| println!("{} closed.", addr)),
			);
			Ok(())
		});

	runtime.block_on(f).unwrap();
}
