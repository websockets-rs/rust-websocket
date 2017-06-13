extern crate websocket;
extern crate futures;
extern crate futures_cpupool;
extern crate tokio_core;

use std::fmt::Debug;

use websocket::message::OwnedMessage;
use websocket::server::InvalidConnection;
use websocket::async::Server;

use tokio_core::reactor::{Handle, Core};

use futures::{Future, Sink, Stream};
use futures::future::{self, Loop};
use futures_cpupool::CpuPool;

use std::time::Duration;
use std::thread;
use std::rc::Rc;

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let server = Server::bind("localhost:8080", &handle).unwrap();
    let pool = Rc::new(CpuPool::new_num_cpus());
    let f = server.incoming()
        // we don't wanna save the stream if it drops
        .map_err(|InvalidConnection { error, .. }| error)
        .for_each(|(upgrade, addr)| {
            println!("Got a connection from: {}", addr);
            if !upgrade.protocols().iter().any(|s| s == "rust-websocket") {
                spawn_future(upgrade.reject(), "Upgrade Rejection", &handle);
                return Ok(());
            }

            let client = upgrade
                .use_protocol("rust-websocket")
                .accept();
            let pool_inner = pool.clone();
            let f = client
                .and_then(move |(framed, _)| {
                    let (sink, stream) = framed.split();

                    let input = stream
                            .for_each(move |msg|{
                                if let OwnedMessage::Text(txt) = msg {
                                    println!("Received message: {}", txt);
                                }
                                Ok(())
                            });

                    // We spawn a new thread for the sender so we can parallely send and receive messages
                    let output = pool_inner.spawn_fn(|| future::loop_fn(sink, move |sink| {
                        // In this example we have an endless sending loop
                        // So we put in a bit of delay to not overload the client
                        thread::sleep(Duration::from_millis(100));
                        sink.send(OwnedMessage::Text("Hello World!".to_owned()))
                            .map(|sink| {
                                match 1 {
                                    1 => Loop::Continue(sink),
                                    _ => Loop::Break(()) // This line is here for type deduction
                                }
                            })
                            .boxed()
                    }));
                    input.join(output)
                });
            spawn_future(f, "Client Status", &handle);
            Ok(())
        });

    core.run(f).unwrap();
}

fn spawn_future<F, I, E>(f: F, desc: &'static str, handle: &Handle)
    where F: Future<Item = I, Error = E> + 'static,
          E: Debug
{
    handle.spawn(f.map_err(move |e| println!("Error in {}: '{:?}'", desc, e))
                     .map(move |_| println!("{}: Finished.", desc)));
}


