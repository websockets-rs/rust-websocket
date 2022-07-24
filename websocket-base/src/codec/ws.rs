//! Send websocket messages and dataframes asynchronously.
//!
//! This module provides codecs that can be be used with `tokio` to create
//! asynchronous streams that can serialize/deserialize websocket messages
//! (and dataframes for users that want low level control).
//!
//! For websocket messages, see the documentation for `MessageCodec`, for
//! dataframes see the documentation for `DataFrameCodec`

extern crate bytes;
extern crate tokio_codec;

use std::borrow::Borrow;
use std::io::Cursor;
use std::marker::PhantomData;
use std::mem;

use self::bytes::BufMut;
use self::bytes::BytesMut;
use self::tokio_codec::Decoder;
use self::tokio_codec::Encoder;

use crate::dataframe::DataFrame;
use crate::message::OwnedMessage;
use crate::result::WebSocketError;
use crate::ws::dataframe::DataFrame as DataFrameTrait;
use crate::ws::message::Message as MessageTrait;
use crate::ws::util::header::read_header;

const DEFAULT_MAX_DATAFRAME_SIZE : usize = 1024*1024*100;
const DEFAULT_MAX_MESSAGE_SIZE : usize = 1024*1024*200;
const MAX_DATAFRAMES_IN_ONE_MESSAGE: usize = 1024*1024;
const PER_DATAFRAME_OVERHEAD : usize = 64;

/// Even though a websocket connection may look perfectly symmetrical
/// in reality there are small differences between clients and servers.
/// This type is passed to the codecs to inform them of what role they are in
/// (i.e. that of a Client or Server).
///
/// For those familiar with the protocol, this decides whether the data should be
/// masked or not.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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

/// A codec for decoding and encoding websocket dataframes.
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
	max_dataframe_size: u32,
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
	/// 
	/// There is a default dataframe size limit imposed. Use `new_with_limits` to override it
	pub fn new(context: Context) -> DataFrameCodec<D> {
		DataFrameCodec::new_with_limits(context, DEFAULT_MAX_DATAFRAME_SIZE)
	}

	pub fn new_with_limits(context: Context, max_dataframe_size: usize) ->  DataFrameCodec<D> {
		let max_dataframe_size: u32 = max_dataframe_size.min(u32::MAX as usize) as u32;
		DataFrameCodec {
			is_server: context == Context::Server,
			frame_type: PhantomData,
			max_dataframe_size,
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

		if header.len > self.max_dataframe_size as u64 {
			return Err(WebSocketError::ProtocolError(
				"Exceeded maximum incoming DataFrame size",
			));
		}

		// check if we have enough bytes to continue
		if header.len + bytes_read > src.len() as u64 {
			return Ok(None);
		}

		// TODO: using usize is not the right thing here (can be larger)
		let _ = src.split_to(bytes_read as usize);
		let body = src.split_to(header.len as usize).to_vec();

		// construct a dataframe
		Ok(Some(DataFrame::read_dataframe_body(
			header,
			body,
			self.is_server,
		)?))
	}
}

impl<D> Encoder for DataFrameCodec<D>
where
	D: Borrow<dyn DataFrameTrait>,
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
///# Example (for the high-level `websocket` crate)
///
///```rust,ignore
///# extern crate tokio;
///# extern crate websocket;
///# extern crate hyper;
///# use std::io::{self, Cursor};
///use websocket::async::{MessageCodec, MsgCodecCtx};
///# use websocket::{Message, OwnedMessage};
///# use websocket::ws::Message as MessageTrait;
///# use websocket::stream::ReadWritePair;
///# use websocket::async::futures::{Future, Sink, Stream};
///# use hyper::http::h1::Incoming;
///# use hyper::version::HttpVersion;
///# use hyper::header::Headers;
///# use hyper::method::Method;
///# use hyper::uri::RequestUri;
///# use hyper::status::StatusCode;
///# use tokio::codec::Decoder;
///# fn main() {
///
///let mut runtime = tokio::runtime::Builder::new().build().unwrap();
///let mut input = Vec::new();
///Message::text("50 schmeckels").serialize(&mut input, false);
///
///let f = MessageCodec::default(MsgCodecCtx::Client)
///    .framed(ReadWritePair(Cursor::new(input), Cursor::new(vec![])))
///    .into_future()
///    .map_err(|e| e.0)
///    .map(|(m, _)| {
///        assert_eq!(m, Some(OwnedMessage::Text("50 schmeckels".to_string())));
///    });
///
///runtime.block_on(f).unwrap();
///# }
pub struct MessageCodec<M>
where
	M: MessageTrait,
{
	buffer: Vec<DataFrame>,
	dataframe_codec: DataFrameCodec<DataFrame>,
	message_type: PhantomData<fn(M)>,
	max_message_size: u32,
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
where
	M: MessageTrait,
{
	/// Creates a codec that can encode a custom implementation of a websocket
	/// message.
	///
	/// If you just want to use a normal codec without a specific implementation
	/// of a websocket message, take a look at `MessageCodec::default`.
	/// 
	/// The codec automatically imposes default limits on message and data frame size.
	/// Use `new_with_limits` to override them.
	pub fn new(context: Context) -> MessageCodec<M> {
		MessageCodec::new_with_limits(context, DEFAULT_MAX_DATAFRAME_SIZE, DEFAULT_MAX_MESSAGE_SIZE)
	}

	pub fn new_with_limits(context: Context, max_dataframe_size: usize, max_message_size: usize) -> MessageCodec<M> {
		let max_message_size: u32 = max_message_size.min(u32::MAX as usize) as u32;
		MessageCodec {
			buffer: Vec::new(),
			dataframe_codec: DataFrameCodec::new_with_limits(context, max_dataframe_size),
			message_type: PhantomData,
			max_message_size,
		}
	}
}

impl<M> Decoder for MessageCodec<M>
where
	M: MessageTrait,
{
	type Item = OwnedMessage;
	type Error = WebSocketError;

	fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
		let mut current_message_length : usize = self.buffer.iter().map(|x|x.data.len()).sum();
		while let Some(frame) = self.dataframe_codec.decode(src)? {
			let is_first = self.buffer.is_empty();
			let finished = frame.finished;

			match frame.opcode as u8 {
				// continuation code
				0 if is_first => {
					return Err(WebSocketError::ProtocolError(
						"Unexpected continuation data frame opcode",
					));
				}
				// control frame
				8..=15 => {
					return Ok(Some(OwnedMessage::from_dataframes(vec![frame])?));
				}
				// data frame
				1..=7 if !is_first => {
					return Err(WebSocketError::ProtocolError(
						"Unexpected data frame opcode",
					));
				}
				// its good
				_ => {
					current_message_length += frame.data.len() + PER_DATAFRAME_OVERHEAD;
					self.buffer.push(frame);
				}
			};

			if finished {
				let buffer = mem::replace(&mut self.buffer, Vec::new());
				return Ok(Some(OwnedMessage::from_dataframes(buffer)?));
			} else {
				if self.buffer.len() >= MAX_DATAFRAMES_IN_ONE_MESSAGE {
					return Err(WebSocketError::ProtocolError(
						"Exceeded count of data frames in one WebSocket message",
					));
				}
				if current_message_length > self.max_message_size as usize {
					return Err(WebSocketError::ProtocolError(
						"Exceeded maximum WebSocket message size",
					));
				}
			}
		}

		Ok(None)
	}
}

impl<M> Encoder for MessageCodec<M>
where
	M: MessageTrait,
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
	extern crate tokio;
	use super::*;
	use crate::message::CloseData;
	use crate::message::Message;
	use crate::stream::ReadWritePair;
	use futures::{Future, Sink, Stream};
	use std::io::Cursor;

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
		let mut input = Vec::new();
		Message::text("50 schmeckels")
			.serialize(&mut input, false)
			.unwrap();

		let f = MessageCodec::new(Context::Client)
			.framed(ReadWritePair(Cursor::new(input), Cursor::new(vec![])))
			.into_future()
			.map_err(|e| e.0)
			.map(|(m, s)| {
				assert_eq!(m, Some(OwnedMessage::Text("50 schmeckels".to_string())));
				s
			})
			.and_then(|s| s.send(Message::text("ethan bradberry")))
			.and_then(|s| {
				let mut stream = s.into_parts().io;
				stream.1.set_position(0);
				println!("buffer: {:?}", stream.1);
				MessageCodec::default(Context::Server)
					.framed(ReadWritePair(stream.1, stream.0))
					.into_future()
					.map_err(|e| e.0)
					.map(|(message, _)| {
						assert_eq!(message, Some(Message::text("ethan bradberry").into()))
					})
			});

		tokio::runtime::Builder::new()
			.build()
			.unwrap()
			.block_on(f)
			.unwrap();
	}

	#[test]
	fn message_codec_server_send_receive() {
		let mut runtime = tokio::runtime::Builder::new().build().unwrap();
		let mut input = Vec::new();
		Message::text("50 schmeckels")
			.serialize(&mut input, true)
			.unwrap();

		let f = MessageCodec::new(Context::Server)
			.framed(ReadWritePair(Cursor::new(input), Cursor::new(vec![])))
			.into_future()
			.map_err(|e| e.0)
			.map(|(m, s)| {
				assert_eq!(m, Some(OwnedMessage::Text("50 schmeckels".to_string())));
				s
			})
			.and_then(|s| s.send(Message::text("ethan bradberry")))
			.map(|s| {
				let mut written = vec![];
				Message::text("ethan bradberry")
					.serialize(&mut written, false)
					.unwrap();
				assert_eq!(written, s.into_parts().io.1.into_inner());
			});

		runtime.block_on(f).unwrap();
	}
}
