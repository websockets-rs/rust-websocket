extern crate websocket;
extern crate futures;
extern crate tokio_core;

use websocket::message::{Message, OwnedMessage};
use websocket::server::InvalidConnection;
use websocket::async::Server;
use websocket::async::client::Client;
use websocket::async::WebSocketFuture;

use tokio_core::reactor::Core;
use tokio_core::net::TcpStream;
use futures::{Future, Sink, Stream};

fn main() {
	let mut core = Core::new().unwrap();
	let handle = core.handle();
	// bind to the server
	let server = Server::bind("127.0.0.1:2794", &handle).unwrap();

	// time to build the server's future
	// this will be a struct containing everything the server is going to do

	// a stream of incoming connections
	let f = server.incoming()
        // we don't wanna save the stream if it drops
        .map_err(|InvalidConnection { error, .. }| error.into())
        // negotiate with the client
        .and_then(|upgrade| {
            // check if it has the protocol we want
            let uses_proto = upgrade.protocols().iter().any(|s| s == "rust-websocket");

            let f: WebSocketFuture<Option<Client<TcpStream>>> = if uses_proto {
                // accept the request to be a ws connection if it does
                Box::new(upgrade.use_protocol("rust-websocket").accept().map(|(s, _)| Some(s)))
            } else {
                // reject it if it doesn't
                Box::new(upgrade.reject().map(|_| None).map_err(|e| e.into()))
            };
            f
        })
        // get rid of the bad connections
        .filter_map(|i| i)
        // send a greeting!
        .and_then(|s| s.send(Message::text("Hello World!").into()))
        // simple echo server impl
        .and_then(|s| {
            let (sink, stream) = s.split();
            stream.filter_map(|m| {
                println!("Message from Client: {:?}", m);
                match m {
                    OwnedMessage::Ping(p) => Some(OwnedMessage::Pong(p)),
                    OwnedMessage::Pong(_) => None,
                    _ => Some(m),
                }
            }).forward(sink)
        })
        // TODO: ??
        .collect();

	core.run(f).unwrap();
}
