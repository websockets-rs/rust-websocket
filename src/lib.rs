#![unstable]
#![warn(missing_docs)]

//! Rust-WebSocket is a WebSocket (RFC6455) library written in Rust.
//! 
//! Rust-WebSocket attempts to provide a framework for WebSocket connections (both clients and servers).
//! The library is currently in an unstable state, but provides all required functionality to implement a WebSocket server or client
//! with the capability to send and receive data frames, messages, and fragmented messages.
//!#Clients
//! To create a WebSocket client, use ```WebSocketRequest::connect()``` and supply a WebSocket URI (e.g. ```ws://127.0.0.1:1234```).
//! Call ```send()``` on that request to retrieve the server's response, and if the response constitutes a success, use
//! ```begin()``` to obtain an actual ```WebSocketClient```.
//! 
//! ```no_run
//!extern crate url;
//!extern crate websocket;
//!# fn main() {
//!use websocket::{WebSocketRequest, WebSocketMessage};
//!use url::Url;
//! 
//!let url = Url::parse("ws://127.0.0.1:1234").unwrap();
//!let request = WebSocketRequest::connect(url).unwrap();
//!let response = request.send().unwrap();
//!let mut client = response.begin();
//! 
//!let message = WebSocketMessage::Text("Hello, server!".to_string());
//!let _ = client.send_message(message);
//! 
//!// ...
//!# }
//! ```
//!#Servers
//! To implement a server, use ```WebSocketServer::bind()``` and supply a socket address (e.g. ```127.0.0.1:1234```).
//! Call ```listen()``` on that server to retrieve a WebSocketAcceptor, ready accepts WebSocket connections.
//! Now call ```WebSocketAcceptor.accept()``` or ```WebSocketAcceptor.incoming()``` to obtain incoming requests.
//! Generate responses using ```WebSocketResponse::new()``` and ```send()``` them to obtain an actual ```WebSocketClient```.
//!
//! ```no_run
//!extern crate websocket;
//!# fn main() {
//!use std::thread::Thread;
//!use std::io::{Listener, Acceptor};
//!use websocket::{WebSocketServer, WebSocketMessage};
//!
//!let server = WebSocketServer::bind("127.0.0.1:1234").unwrap();
//!let mut acceptor = server.listen().unwrap();
//!for request in acceptor.incoming() {
//!    Thread::spawn(move || {
//!        let request = request.unwrap();
//!        let response = request.accept();
//!        let mut client = response.send().unwrap();
//!        
//!        let message = WebSocketMessage::Text("Hello, client!".to_string());
//!        let _ = client.send_message(message);
//!        
//!        // ...
//!    }).detach();
//!}
//!# }
//! ```
//! 
//!#Concurrency and Safety
//! Rust-WebSocket employs a Mutex to provide a locking system on the Reader and Writer
//! which constitute a WebSocketClient. This means you can safely send and receive messages
//! (but not necessarily data frames) from different threads, while ensuring the WebSocket
//! stream does not become corrupted. Use the ```WebSocketClient.clone()``` method to clone
//! the client, allowing it to be moved into another thread.
//! 
//! The functions ```frag_send_text()``` and ```frag_send_bytes()``` can also safely be used
//! as they lock the Writer for the duration of their use.
//! 
//! Avoid use of ```send_dataframe()``` and ```recv_dataframe()```  unless lower level
//! control is needed, as they allow the WebSocket stream to become corrupted (through
//! message interleaving, etc).
//!
//!```no_run
//!# extern crate url;
//!# extern crate websocket;
//!# fn main() {
//!use websocket::{WebSocketRequest, WebSocketMessage};
//!use url::Url;
//!use std::thread::Thread;
//!# let url = Url::parse("ws://127.0.0.1:1234").unwrap();
//!# let request = WebSocketRequest::connect(url).unwrap();
//!# let response = request.send().unwrap();
//!let mut client_send = response.begin(); // Use this object to send
//!let mut client_receive = client_send.clone(); // Use this object to receive
//!
//!Thread::spawn(move || { // Move client_send to a new thread
//!    let message = WebSocketMessage::Text("Hello, world!".to_string());
//!    let _ = client_send.send_message(message);
//!}).detach();
//!
//!for message in client_receive.incoming_messages() {
//!    match message.unwrap() {
//!        WebSocketMessage::Text(text) => { println!("Text: {}", text); },
//!        WebSocketMessage::Binary(data) => { println!("Binary data received"); },
//!        _ => { }
//!    }
//!}
//!# }
//!```
//!#Extending Rust-WebSocket
//! Rust-WebSocket provides a number of mechanisms to extend the WebSocket implementation.
//! 
//! The function ```WebSocketRequest::connect_with()``` gives the ability to use a different Reader and Writer,
//! The method ```WebSocketResponse::begin_with()``` allows the use of any WebSocketDataFrameSender, WebSocketDataFrameReceiver,
//! WebSocketDataFrameConverter and WebSocketMessaging trait implementers. These traits provide interfaces required for use
//! within Rust-WebSocket.
//! 
//! See each trait for more detailed information.
extern crate hyper;
extern crate url;
extern crate "rustc-serialize" as serialize;
extern crate sha1;
extern crate openssl;

pub use self::client::WebSocketClient;
pub use self::handshake::request::WebSocketRequest;
pub use self::handshake::response::WebSocketResponse;
pub use self::server::WebSocketServer;
pub use self::message::WebSocketMessage;

pub mod client;
pub mod common;
pub mod dataframe;
pub mod handshake;
pub mod header;
pub mod message;
pub mod server;
