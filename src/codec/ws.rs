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

/**************
 * Dataframes *
 **************/

#[derive(Clone,PartialEq,Eq,Debug)]
pub enum Context {
	Server,
	Client,
}

pub struct DataFrameCodec<D> {
	is_server: bool,
	frame_type: PhantomData<D>,
}

impl DataFrameCodec<DataFrame> {
	pub fn default(context: Context) -> Self {
		DataFrameCodec::new(context)
	}
}

impl<D> DataFrameCodec<D> {
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

pub struct MessageCodec<M>
	where M: MessageTrait
{
	buffer: Vec<DataFrame>,
	dataframe_codec: DataFrameCodec<DataFrame>,
	message_type: PhantomData<fn(M)>,
}

impl MessageCodec<OwnedMessage> {
	pub fn default(context: Context) -> Self {
		Self::new(context)
	}
}

impl<M> MessageCodec<M>
    where M: MessageTrait
{
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
					return Err(WebSocketError::ProtocolError("Unexpected continuation data frame opcode"));
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

// TODO: add tests to check boundary cases for reading dataframes
