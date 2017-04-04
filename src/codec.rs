//! TODO: docs

use std::borrow::Borrow;
use std::marker::PhantomData;
use std::io::Cursor;
use std::mem;
use std::io;

use tokio_io::codec::Decoder;
use tokio_io::codec::Encoder;
use bytes::BytesMut;
use bytes::BufMut;

use dataframe::DataFrame;
use message::Message;
use ws::dataframe::DataFrame as DataFrameTrait;
use ws::message::Message as MessageTrait;
use ws::util::header::read_header;
use result::WebSocketError;

/**************
 * Dataframes *
 **************/

/// TODO: docs
pub struct DataFrameCodec<D> {
	masked: bool,
	frame_type: PhantomData<D>,
}

impl<D> DataFrameCodec<D> {
	/// TODO: docs
	pub fn default(masked: bool) -> DataFrameCodec<DataFrame> {
		DataFrameCodec::new(masked)
	}

	/// TODO: docs
	pub fn new(masked: bool) -> DataFrameCodec<D> {
		DataFrameCodec {
			masked: masked,
			frame_type: PhantomData,
		}
	}
}

impl<D> Decoder for DataFrameCodec<D> {
	type Item = DataFrame;
	type Error = WebSocketError;

	fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
		let (header, bytes_read) = {
			// we'll make a fake reader and keep track of the bytes read
			let mut reader = Cursor::new(src.as_ref());

			// read header to get the size, bail if not enough
			let header = match read_header(&mut reader) {
				Ok(head) => head,
				// TODO: check if this is the correct error
				Err(WebSocketError::IoError(ref e)) if e.kind() == io::ErrorKind::UnexpectedEof => {
					return Ok(None)
				}
				Err(e) => return Err(e),
			};

			(header, reader.position())
		};

		// check if we have enough bytes to continue
		if header.len + bytes_read > src.len() as u64 {
			return Ok(None);
		}

		let body = src.split_off(bytes_read as usize).to_vec();
		// use up the rest of the buffer since we already copied it to header
		let _ = src.take();

		// construct a dataframe
		Ok(Some(DataFrame::read_dataframe_body(header, body, self.masked)?))
	}
}

// TODO: try to allow encoding of many kinds of dataframes
impl<D> Encoder for DataFrameCodec<D>
    where D: Borrow<DataFrameTrait>
{
	type Item = D;
	type Error = WebSocketError;

	fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
		// TODO: check size and grow dst accordingly
		item.borrow().write_to(&mut dst.writer(), self.masked)
	}
}

/************
 * Messages *
 ************/

/// TODO: docs
pub struct MessageCodec<'m, M>
	where M: 'm
{
	buffer: Vec<DataFrame>,
	dataframe_codec: DataFrameCodec<DataFrame>,
	message_type: PhantomData<fn(&'m M)>,
}

impl<'m, M> MessageCodec<'m, M> {
	/// TODO: docs
	pub fn default(masked: bool) -> MessageCodec<'m, Message<'m>> {
		MessageCodec::new(masked)
	}

	/// TODO: docs
	pub fn new(masked: bool) -> MessageCodec<'m, M> {
		MessageCodec {
			buffer: Vec::new(),
			dataframe_codec: DataFrameCodec {
				masked: masked,
				frame_type: PhantomData,
			},
			message_type: PhantomData,
		}
	}
}

impl<'m, M> Decoder for MessageCodec<'m, M>
    where M: 'm
{
	type Item = Message<'m>;
	type Error = WebSocketError;

	fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
		while let Some(frame) = self.dataframe_codec.decode(src)? {
			let is_first = self.buffer.is_empty();
			let finished = frame.finished;

			match frame.opcode as u8 {
				// continuation code
				0 if is_first => {
					return Err(WebSocketError::ProtocolError("Unexpected continuation data frame opcode"));
				}
				// control frame
				8...15 => {
					mem::replace(&mut self.buffer, vec![frame]);
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
				return Ok(Some(Message::from_dataframes(buffer)?));
			}
		}

		Ok(None)
	}
}

impl<'m, M> Encoder for MessageCodec<'m, M>
    where M: Borrow<Message<'m>>
{
	type Item = M;
	type Error = WebSocketError;

	fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
		for ref dataframe in item.borrow().dataframes() {
			// TODO: check size and grow dst accordingly
			dataframe.write_to(&mut dst.writer(), self.dataframe_codec.masked)?
		}
		Ok(())
	}
}

// TODO: add tests to check boundary cases for reading dataframes
