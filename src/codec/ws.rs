//! Send websocket messages and dataframes asynchronously.
//!
//! This module provides codecs that can be be used with `tokio` to create
//! asynchronous streams that can serialize/deserialize websocket messages
//! (and dataframes for users that want low level control).
//!
//! For websocket messages, see the documentation for `MessageCodec`, for
//! dataframes see the documentation for `DataFrameCodec`

use std::borrow::Borrow;
use std::marker::PhantomData;
use std::io::Cursor;
use std::mem;

use tokio_io::codec::Decoder;
use tokio_io::codec::Encoder;
use bytes::BytesMut;
use bytes::BufMut;

use dataframe::DataFrame;
use message::OwnedMessage;
use ws::dataframe::DataFrame as DataFrameTrait;
use ws::message::Message as MessageTrait;
use ws::util::header::read_header;
use result::WebSocketError;

/// Even though a websocket connection may look perfectly symmetrical
/// in reality there are small differences between clients and servers.
/// This type is passed to the codecs to inform them of what role they are in
/// (i.e. that of a Client or Server).
///
/// For those familiar with the protocol, this decides wether the data should be
/// masked or not.
#[derive(Clone,PartialEq,Eq,Debug)]
pub enum Context {
	/// Set the codec to act in `Server` mode, used when
	/// implementing a websocket server.
	Server,
	/// Set the codec to act in `Client` mode, used when
	/// implementing a websocket client.
	Client,
}

/**************
 * Dataframes *
 **************/

/// A codec for deconding and encoding websocket dataframes.
///
/// This codec decodes dataframes into the crates default implementation
/// of `Dataframe` but can encode and send any struct that implements the
/// `ws::Dataframe` trait. The type of struct to encode is given by the `D`
/// type parameter in the struct.
///
/// Using dataframes directly is meant for users who want low-level access to the
/// connection. If you don't want to do anything low-level please use the
/// `MessageCodec` codec instead, or better yet use the `ClientBuilder` to make
/// clients and the `Server` to make servers.
pub struct DataFrameCodec<D> {
	is_server: bool,
	frame_type: PhantomData<D>,
}

impl DataFrameCodec<DataFrame> {
	/// Create a new `DataFrameCodec` struct using the crate's implementation
	/// of dataframes for reading and writing dataframes.
	///
	/// Use this method if you don't want to provide a custom implementation
	/// for your dataframes.
	pub fn default(context: Context) -> Self {
		DataFrameCodec::new(context)
	}
}

impl<D> DataFrameCodec<D> {
	/// Create a new `DataFrameCodec` struct using any implementation of
	/// `ws::Dataframe` you want. This is useful if you want to manipulate
	/// the websocket layer very specifically.
	///
	/// If you only want to be able to send and receive the crate's
	/// `DataFrame` struct use `.default(Context)` instead.
	pub fn new(context: Context) -> DataFrameCodec<D> {
		DataFrameCodec {
			is_server: context == Context::Server,
			frame_type: PhantomData,
		}
	}
}

impl<D> Decoder for DataFrameCodec<D> {
	type Item = DataFrame;
	type Error = WebSocketError;

	// TODO: do not retry to read the header on each new data (keep a buffer)
	fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
		let (header, bytes_read) = {
			// we'll make a fake reader and keep track of the bytes read
			let mut reader = Cursor::new(src.as_ref());

			// read header to get the size, bail if not enough
			let header = match read_header(&mut reader) {
				Ok(head) => head,
				Err(WebSocketError::NoDataAvailable) => return Ok(None),
				Err(e) => return Err(e),
			};

			(header, reader.position())
		};

		// check if we have enough bytes to continue
		if header.len + bytes_read > src.len() as u64 {
			return Ok(None);
		}

		// TODO: using usize is not the right thing here (can be larger)
		let _ = src.split_to(bytes_read as usize);
		let body = src.split_to(header.len as usize).to_vec();

		// construct a dataframe
		Ok(Some(DataFrame::read_dataframe_body(header, body, self.is_server)?))
	}
}

impl<D> Encoder for DataFrameCodec<D>
    where D: Borrow<DataFrameTrait>
{
	type Item = D;
	type Error = WebSocketError;

	fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
		let masked = !self.is_server;
		let frame_size = item.borrow().frame_size(masked);
		if frame_size > dst.remaining_mut() {
			dst.reserve(frame_size);
		}
		item.borrow().write_to(&mut dst.writer(), masked)
	}
}

/************
 * Messages *
 ************/

/// A codec for asynchronously decoding and encoding websocket messages.
///
/// This codec decodes messages into the `OwnedMessage` struct, so using this
/// the user will receive messages as `OwnedMessage`s. However it can encode
/// any type of message that implements the `ws::Message` trait (that type is
/// decided by the `M` type parameter) like `OwnedMessage` and `Message`.
///
/// Warning: if you don't know what your doing or want a simple websocket connection
/// please use the `ClientBuilder` or the `Server` structs. You should only use this
/// after a websocket handshake has already been completed on the stream you are
/// using.
///
///# Example
///
///```rust
///# extern crate tokio_core;
///# extern crate tokio_io;
///# extern crate websocket;
///# extern crate hyper;
///# use std::io::{self, Cursor};
///use websocket::async::{MessageCodec, MsgCodecCtx};
///# use websocket::{Message, OwnedMessage};
///# use websocket::ws::Message as MessageTrait;
///# use websocket::stream::ReadWritePair;
///# use websocket::async::futures::{Future, Sink, Stream};
///# use tokio_core::net::TcpStream;
///# use tokio_core::reactor::Core;
///# use tokio_io::AsyncRead;
///# use hyper::http::h1::Incoming;
///# use hyper::version::HttpVersion;
///# use hyper::header::Headers;
///# use hyper::method::Method;
///# use hyper::uri::RequestUri;
///# use hyper::status::StatusCode;
///# fn main() {
///
///let mut core = Core::new().unwrap();
///let mut input = Vec::new();
///Message::text("50 schmeckels").serialize(&mut input, false);
///
///let f = ReadWritePair(Cursor::new(input), Cursor::new(vec![]))
///    .framed(MessageCodec::default(MsgCodecCtx::Client))
///    .into_future()
///    .map_err(|e| e.0)
///    .map(|(m, _)| {
///        assert_eq!(m, Some(OwnedMessage::Text("50 schmeckels".to_string())));
///    });
///
///core.run(f).unwrap();
///# }
pub struct MessageCodec<M>
	where M: MessageTrait
{
	buffer: Vec<DataFrame>,
	dataframe_codec: DataFrameCodec<DataFrame>,
	message_type: PhantomData<fn(M)>,
}

impl MessageCodec<OwnedMessage> {
	/// Create a new `MessageCodec` with a role of `context` (either `Client`
	/// or `Server`) to read and write messages asynchronously.
	///
	/// This will create the crate's default codec which sends and receives
	/// `OwnedMessage` structs. The message data has to be sent to an intermediate
	/// buffer anyway so sending owned data is preferable.
	///
	/// If you have your own implementation of websocket messages, you can
	/// use the `new` method to create a codec for that implementation.
	pub fn default(context: Context) -> Self {
		Self::new(context)
	}
}

impl<M> MessageCodec<M>
    where M: MessageTrait
{
	/// Creates a codec that can encode a custom implementation of a websocket
	/// message.
	///
	/// If you just want to use a normal codec without a specific implementation
	/// of a websocket message, take a look at `MessageCodec::default`.
	pub fn new(context: Context) -> MessageCodec<M> {
		MessageCodec {
			buffer: Vec::new(),
			dataframe_codec: DataFrameCodec::new(context),
			message_type: PhantomData,
		}
	}
}

impl<M> Decoder for MessageCodec<M>
    where M: MessageTrait
{
	type Item = OwnedMessage;
	type Error = WebSocketError;

	fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
		while let Some(frame) = self.dataframe_codec.decode(src)? {
			let is_first = self.buffer.is_empty();
			let finished = frame.finished;

			match frame.opcode as u8 {
				// continuation code
				0 if is_first => {
					return Err(WebSocketError::ProtocolError("Unexpected continuation data frame opcode",),);
				}
				// control frame
				8...15 => {
					return Ok(Some(OwnedMessage::from_dataframes(vec![frame])?));
				}
				// data frame
				1...7 if !is_first => {
					return Err(WebSocketError::ProtocolError("Unexpected data frame opcode"));
				}
				// its good
				_ => {
					self.buffer.push(frame);
				}
			};

			if finished {
				let buffer = mem::replace(&mut self.buffer, Vec::new());
				return Ok(Some(OwnedMessage::from_dataframes(buffer)?));
			}
		}

		Ok(None)
	}
}

impl<M> Encoder for MessageCodec<M>
    where M: MessageTrait
{
	type Item = M;
	type Error = WebSocketError;

	fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
		let masked = !self.dataframe_codec.is_server;
		let frame_size = item.message_size(masked);
		if frame_size > dst.remaining_mut() {
			dst.reserve(frame_size);
		}
		item.serialize(&mut dst.writer(), masked)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use tokio_io::AsyncRead;
	use tokio_core::reactor::Core;
	use futures::{Stream, Sink, Future};
	use std::io::Cursor;
	use stream::ReadWritePair;
	use message::CloseData;
	use message::Message;

	#[test]
	fn owned_message_predicts_size() {
		let messages = vec![
			OwnedMessage::Text("nilbog".to_string()),
			OwnedMessage::Binary(vec![1, 2, 3, 4]),
			OwnedMessage::Binary(vec![42; 256]),
			OwnedMessage::Binary(vec![42; 65535]),
			OwnedMessage::Binary(vec![42; 65555]),
			OwnedMessage::Ping("beep".to_string().into_bytes()),
			OwnedMessage::Pong("boop".to_string().into_bytes()),
			OwnedMessage::Close(None),
			OwnedMessage::Close(Some(CloseData {
			                             status_code: 64,
			                             reason: "because".to_string(),
			                         })),
		];

		for message in messages.into_iter() {
			let masked_predicted = message.message_size(true);
			let mut masked_buf = Vec::new();
			message.serialize(&mut masked_buf, true).unwrap();
			assert_eq!(masked_buf.len(), masked_predicted);

			let unmasked_predicted = message.message_size(false);
			let mut unmasked_buf = Vec::new();
			message.serialize(&mut unmasked_buf, false).unwrap();
			assert_eq!(unmasked_buf.len(), unmasked_predicted);
		}
	}

	#[test]
	fn cow_message_predicts_size() {
		let messages = vec![
			Message::binary(vec![1, 2, 3, 4]),
			Message::binary(vec![42; 256]),
			Message::binary(vec![42; 65535]),
			Message::binary(vec![42; 65555]),
			Message::text("nilbog".to_string()),
			Message::ping("beep".to_string().into_bytes()),
			Message::pong("boop".to_string().into_bytes()),
			Message::close(),
			Message::close_because(64, "because"),
		];

		for message in messages.iter() {
			let masked_predicted = message.message_size(true);
			let mut masked_buf = Vec::new();
			message.serialize(&mut masked_buf, true).unwrap();
			assert_eq!(masked_buf.len(), masked_predicted);

			let unmasked_predicted = message.message_size(false);
			let mut unmasked_buf = Vec::new();
			message.serialize(&mut unmasked_buf, false).unwrap();
			assert_eq!(unmasked_buf.len(), unmasked_predicted);
		}
	}

	#[test]
	fn message_codec_client_send_receive() {
		let mut core = Core::new().unwrap();
		let mut input = Vec::new();
		Message::text("50 schmeckels").serialize(&mut input, false).unwrap();

		let f = ReadWritePair(Cursor::new(input), Cursor::new(vec![]))
			.framed(MessageCodec::new(Context::Client))
			.into_future()
			.map_err(|e| e.0)
			.map(|(m, s)| {
				     assert_eq!(m, Some(OwnedMessage::Text("50 schmeckels".to_string())));
				     s
				    })
			.and_then(|s| s.send(Message::text("ethan bradberry")))
			.and_then(|s| {
				let mut stream = s.into_parts().inner;
				stream.1.set_position(0);
				println!("buffer: {:?}", stream.1);
				ReadWritePair(stream.1, stream.0)
					.framed(MessageCodec::default(Context::Server))
					.into_future()
					.map_err(|e| e.0)
					.map(|(message, _)| {
						     assert_eq!(message, Some(Message::text("ethan bradberry").into()))
						    })
			});

		core.run(f).unwrap();
	}

	#[test]
	fn message_codec_server_send_receive() {
		let mut core = Core::new().unwrap();
		let mut input = Vec::new();
		Message::text("50 schmeckels").serialize(&mut input, true).unwrap();

		let f = ReadWritePair(Cursor::new(input.as_slice()), Cursor::new(vec![]))
			.framed(MessageCodec::new(Context::Server))
			.into_future()
			.map_err(|e| e.0)
			.map(|(m, s)| {
				     assert_eq!(m, Some(OwnedMessage::Text("50 schmeckels".to_string())));
				     s
				    })
			.and_then(|s| s.send(Message::text("ethan bradberry")))
			.map(|s| {
				     let mut written = vec![];
				     Message::text("ethan bradberry").serialize(&mut written, false).unwrap();
				     assert_eq!(written, s.into_parts().inner.1.into_inner());
				    });

		core.run(f).unwrap();
	}
}
