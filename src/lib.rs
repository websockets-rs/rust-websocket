#![unstable]
#![warn(missing_docs)]
#![allow(unstable)]
#![feature(associated_types, old_impl_check)]

//! Rust-WebSocket is a WebSocket (RFC6455) library written in Rust.
//! 
//! Rust-WebSocket attempts to provide a framework for WebSocket connections (both clients and servers).
//! The library is currently in an unstable state, but provides all required functionality to implement a WebSocket server or client
//! with the capability to send and receive data frames, messages, and fragmented messages.
//!
//! Find this project on [GitHub](https://github.com/cyderize/rust-websocket) or [crates.io](https://crates.io/crates/websocket).
//!
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
//! See the documentation for WebSocketRequest for more information on how requests are sent.
//!
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
//!    });
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
//!});
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
//! The method ```WebSocketResponse::begin_with()``` allows the use of any DataFrameSender, DataFrameReceiver,
//! DataFrameConverter and WebSocketMessaging trait implementers. These traits provide interfaces required for use
//! within Rust-WebSocket.
//! 
//! See each trait for more detailed information.
//!
//! The most important type is the WebSocketClient, which wraps a DataFrameSender, DataFrameReceiver and DataFrameConverter.
//! The sender, receiver and converter provide the underlying functionality required for the client.
//!
//! A DataFrameSender wraps a Writer, which is generally a WebSocket stream on to which data is written. The DataFrameSender must
//! provide a means of instantiation with ```new()``` and a way to write a WebSocketDataFrame with ```send_dataframe()```.
//!
//! A DataFrameReceiver wraps a Reader, which is generally a WebSocket stream off of which data is read. The DataFrameReceiver
//! must provid a means of instantiation with ```new()``` and a way to read a WebSocketDataFrame with ```recv_dataframe()```.
//!
//! The RawDataFrame provides a convenient middle layer between a stream and a WebSocketDataFrame. A RawDataFrame contains
//! the complete structure of an RFC6455 data frame. The ```read()``` and ```write()``` methods allow a DataFrameSender or
//! DataFrameReceiver to simply provide the underlying stream to the RawDataFrame and let it do the job of writing or reading.
//! Then, the DataFrameSender and DataFrameReceiver only need to provide conversion between WebSocketDataFrames and RawDataFrame.
//!
//! A DataFrameConverter takes WebSocketDataFrames and produces messages. A message needs to implement the WebSocketMessaging trait,
//! which allows for the creation of a message using a WebSocketOpcode and binary data, as well as the conversion of the message
//! into a dataframe.
//!
//! These traits are unstable and subject to change at any time. In the future, the WebSocketDataFrame is likely to be made into a trait
//! to better allow for custom extensions to the WebSocket protocol. In addition, the WebSocketMessaging trait will likely allow messages
//! to produce multiple data frames from themselves.

extern crate hyper;
extern crate url;
extern crate "rustc-serialize" as serialize;
extern crate "sha1-hasher" as sha1;
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
