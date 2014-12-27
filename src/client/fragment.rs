//! Helper structs for dealing with WebSocket fragmented messages
//!
//! The functions ```string_fragmenter()``` and ```binary_fragmenter()``` provide a tuple of
//! a writer and an iterator. The data is written to the writer, and the iterator is passed to
//! ```WebSocketClient.frag_send_text()``` or ```WebSocketClient.frag_send_bytes()```. Since
//! the ```frag_send``` functions can only lock the stream for the scope of the function, they
//! must be kept running *while* the data is being written to the writer. That is, the call to
//! the ```frag_send``` function should happen on a different thread to which the data is being
//! written on. See the examples for each function.
#![unstable]

/// A helper struct for sending fragmented strings.
/// 
/// Returns a tuple consisting of a writer (to write the strings to)
/// and an iterator (to use with WebSocketClient.frag_send_text().
/// 
/// ```no_run
///# extern crate url;
///# extern crate websocket;
///# fn main() {
///# use websocket::WebSocketRequest;
///# use url::Url;
///use websocket::client::fragment::string_fragmenter;
///use std::thread::Thread;
///# let url = Url::parse("ws://127.0.0.1:1234").unwrap();
///# let request = WebSocketRequest::connect(url).unwrap();
///# let response = request.send().unwrap();
///# let mut client = response.begin();
///let (mut writer, iterator) = string_fragmenter(); //Returns a writer and an iterator
///// We write our data to the writer in another thread:
///Thread::spawn(move || {
///    writer.push("This ");
///    writer.push("is ");
///    writer.push("a ");
///    writer.push("fragmented ");
///    writer.push("message.");
///    writer.finish();			
///}).detach();
///// Immediately starts sending the data
///let _ = client.frag_send_text(iterator);
///# }
/// ```
#[unstable]
pub fn string_fragmenter() -> (StringFragmentWriter, StringFragmentIterator) {
	let (tx, rx) = channel();
	(StringFragmentWriter { inner: tx }, StringFragmentIterator{ inner: rx })
}

/// A helper struct for sending fragmented strings.
/// Write the strings to this object using StringFragmentWriter::push().
#[unstable]
pub struct StringFragmentWriter {
	inner: Sender<Option<String>>,
}

/// A helper struct for sending fragmented strings.
/// Use with WebSocketClient.frag_send_text().
#[unstable]
pub struct StringFragmentIterator {
	inner: Receiver<Option<String>>,
}

impl StringFragmentWriter {
	/// Push a string on to this writer
	#[unstable]
	pub fn push<S: ToString>(&mut self, string: S) {
		self.inner.send(Some(string.to_string()));
	}
	/// Signal the end of a message
	#[unstable]
	pub fn finish(&mut self) {
		self.inner.send(None);
	}
}

impl Iterator<String> for StringFragmentIterator {
	fn next(&mut self) -> Option<String> {
		self.inner.recv()
	}
}

/// A helper struct for sending fragmented binary data.
/// 
/// Returns a tuple consisting of a writer (to write the data to)
/// and an iterator (to use with WebSocketClient.frag_send_bytes().
/// 
/// ```no_run
///# extern crate url;
///# extern crate websocket;
///# fn main() {
///# use websocket::WebSocketRequest;
///# use url::Url;
///use websocket::client::fragment::binary_fragmenter;
///use std::thread::Thread;
///# let url = Url::parse("ws://127.0.0.1:1234").unwrap();
///# let request = WebSocketRequest::connect(url).unwrap();
///# let response = request.send().unwrap();
///# let mut client = response.begin();
///let (mut writer, iterator) = binary_fragmenter(); //Returns a writer and an iterator
///// We write our data to the writer in another thread:
///Thread::spawn(move || {
///    writer.push(b"This ");
///    writer.push(b"is ");
///    writer.push(b"a ");
///    writer.push(b"binary ");
///    writer.push(b"message.");
///    writer.finish();			
///}).detach();
///// Immediately starts sending the data
///let _ = client.frag_send_bytes(iterator);
///# }
/// ```
#[unstable]
pub fn binary_fragmenter() -> (BinaryFragmentWriter, BinaryFragmentIterator) {
	let (tx, rx) = channel();
	(BinaryFragmentWriter { inner: tx }, BinaryFragmentIterator{ inner: rx })
}

/// A helper struct for sending fragmented binary data.
/// Write the data to this object using BinaryFragmentWriter::push().
#[unstable]
pub struct BinaryFragmentWriter {
	inner: Sender<Option<Vec<u8>>>,
}

/// A helper struct for sending fragmented binary data.
/// Use with WebSocketClient.frag_send_bytes().
#[unstable]
pub struct BinaryFragmentIterator {
	inner: Receiver<Option<Vec<u8>>>,
}

impl BinaryFragmentWriter {
	/// Push binary data on to this writer
	#[unstable]
	pub fn push(&mut self, data: &[u8]) {
		self.inner.send(Some(Vec::new() + data));
	}
	/// Signal the end of a message
	#[unstable]
	pub fn finish(&mut self) {
		self.inner.send(None);
	}
}

impl Iterator<Vec<u8>> for BinaryFragmentIterator {
	fn next(&mut self) -> Option<Vec<u8>> {
		self.inner.recv()
	}
}