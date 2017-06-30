extern crate websocket;
extern crate futures;
extern crate futures_cpupool;
extern crate tokio_core;

use websocket::message::OwnedMessage;
use websocket::server::InvalidConnection;
use websocket::async::Server;

use tokio_core::reactor::{Handle, Remote, Core};

use futures::{Future, Sink, Stream};
use futures::future::{self, Loop};
use futures::sync::mpsc;
use futures_cpupool::CpuPool;

use std::sync::{RwLock, Arc};
use std::thread;
use std::rc::Rc;
use std::time::Duration;
use std::collections::HashMap;
use std::cell::RefCell;
use std::fmt::Debug;

type Id = u32;

fn main() {
	let mut core = Core::new().expect("Failed to create Tokio event loop");
	let handle = core.handle();
	let remote = core.remote();
	let server = Server::bind("localhost:8081", &handle).expect("Failed to create server");
	let pool = Rc::new(CpuPool::new_num_cpus());
	let connections = Arc::new(RwLock::new(HashMap::new()));
	let (receive_channel_out, receive_channel_in) = mpsc::unbounded();

	let conn_id = Rc::new(RefCell::new(Counter::new()));
	let connections_inner = connections.clone();
	// Handle new connection
	let connection_handler =
		server.incoming()
        // we don't wanna save the stream if it drops
        .map_err(|InvalidConnection { error, .. }| error)
        .for_each(move |(upgrade, addr)| {
            let connections_inner = connections_inner.clone();
            println!("Got a connection from: {}", addr);
            let channel = receive_channel_out.clone();
            let handle_inner = handle.clone();
            let conn_id = conn_id.clone();
            let f = upgrade
                .accept()
                .and_then(move |(framed, _)| {
                    let id = conn_id
                        .borrow_mut()
                        .next()
                        .expect("maximum amount of ids reached");
                    let (sink, stream) = framed.split();
                    let f = channel.send((id, stream));
                    spawn_future(f, "Senk stream to connection pool", &handle_inner);
                    connections_inner.write().unwrap().insert(id, sink);
                    Ok(())
                });
            spawn_future(f, "Handle new connection", &handle);
            Ok(())
        })
        .map_err(|_| ());


	// Handle receiving messages from a client
	let remote_inner = remote.clone();
	let receive_handler = pool.spawn_fn(|| {
		receive_channel_in.for_each(move |(id, stream)| {
			remote_inner.spawn(move |_| {
				                   stream.for_each(move |msg| {
					                                   process_message(id, &msg);
					                                   Ok(())
					                                  })
				                         .map_err(|_| ())
				                  });
			Ok(())
		})
	});

	let (send_channel_out, send_channel_in) = mpsc::unbounded();

	// Handle sending messages to a client
	let connections_inner = connections.clone();
	let remote_inner = remote.clone();
	let send_handler = pool.spawn_fn(move || {
		let connections = connections_inner.clone();
		let remote = remote_inner.clone();
		send_channel_in.for_each(move |(id, msg): (Id, String)| {
			let connections = connections.clone();
			let sink = connections.write()
			                      .unwrap()
			                      .remove(&id)
			                      .expect("Tried to send to invalid client id",);

			println!("Sending message '{}' to id {}", msg, id);
			let f = sink.send(OwnedMessage::Text(msg))
			            .and_then(move |sink| {
				                      connections.write().unwrap().insert(id, sink);
				                      Ok(())
				                     });
			remote.spawn(move |_| f.map_err(|_| ()));
			Ok(())
		})
		               .map_err(|_| ())
	});

	// Main 'logic' loop
	let main_loop = pool.spawn_fn(move || {
		future::loop_fn(send_channel_out, move |send_channel_out| {
			thread::sleep(Duration::from_millis(100));

			let should_continue = update(connections.clone(), send_channel_out.clone(), &remote);
			match should_continue {
				Ok(true) => Ok(Loop::Continue(send_channel_out)),
				Ok(false) => Ok(Loop::Break(())),
				Err(()) => Err(()),
			}
		})
	});

	let handlers =
		main_loop.select2(connection_handler.select2(receive_handler.select(send_handler)));
	core.run(handlers).map_err(|_| println!("Error while running core loop")).unwrap();
}

fn spawn_future<F, I, E>(f: F, desc: &'static str, handle: &Handle)
	where F: Future<Item = I, Error = E> + 'static,
	      E: Debug
{
	handle.spawn(f.map_err(move |e| println!("Error in {}: '{:?}'", desc, e))
	              .map(move |_| println!("{}: Finished.", desc)));
}


fn process_message(id: u32, msg: &OwnedMessage) {
	if let OwnedMessage::Text(ref txt) = *msg {
		println!("Received message '{}' from id {}", txt, id);
	}
}

type SinkContent = websocket::client::async::Framed<tokio_core::net::TcpStream,
                                                    websocket::async::MessageCodec<OwnedMessage>>;
type SplitSink = futures::stream::SplitSink<SinkContent>;
// Represents one tick in the main loop
fn update(
	connections: Arc<RwLock<HashMap<Id, SplitSink>>>,
	channel: mpsc::UnboundedSender<(Id, String)>,
	remote: &Remote,
) -> Result<bool, ()> {
	remote.spawn(move |handle| {
		             for (id, _) in connections.read().unwrap().iter() {
			             let f = channel.clone().send((*id, "Hi there!".to_owned()));
			             spawn_future(f, "Send message to write handler", handle);
			            }
		             Ok(())
		            });
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
