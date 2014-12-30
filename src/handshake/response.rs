#![unstable]
//! Structs for WebSocket responses
use std::option::Option;
use hyper::status::StatusCode;
use hyper::version::HttpVersion;
use hyper::header::Headers;
use hyper::header::common::{Connection, Upgrade};
use hyper::header::connection::ConnectionOption;
use hyper::header::common::upgrade::Protocol;
use hyper::http::read_status_line;
use header::{WebSocketKey, WebSocketAccept, WebSocketProtocol, WebSocketExtensions};
use client::WebSocketClient;
use common::{Inbound, Outbound, Local, Remote, WebSocketResult, WebSocketError};
use message::{WebSocketMessaging, WebSocketMessage};
use dataframe::sender::{DataFrameSender, WebSocketSender};
use dataframe::receiver::{DataFrameReceiver, WebSocketReceiver};
use dataframe::converter::{DataFrameConverter, WebSocketConverter};

/// Represents a WebSocket response
/// 
/// A WebSocketResponse can sent by a WebSocket server by calling ```accept()``` on the WebSocketRequest 
/// and then ```send()``` on the response, or received by a WebSocket client to be consumed into a
/// WebSocketClient with ```begin()```.
/// 
/// A type parameter that is either Inbound or Outbound ensures that only the appropriate methods
/// can be called on either kind of request.
/// 
/// The headers field allows access to the HTTP headers in the response, but short-cut methods are
/// available for accessing common headers.
pub struct WebSocketResponse<R: Reader, W: Writer, B> {
	/// The status of the response
	pub status: StatusCode,
	/// The headers contained in this response
	pub headers: Headers,
	/// The HTTP version of this response
	pub version: HttpVersion,
	
	reader: R,
	writer: W,
}

impl<R: Reader + Send, W: Writer + Send, B> WebSocketResponse<R, W, B> {
	/// Return the inner Reader and Writer
	pub fn into_inner(self) -> (R, W) {
		(self.reader, self.writer)
	}
	/// Short-cut to obtain the WebSocketAccept value
	pub fn accept(&self) -> Option<&WebSocketAccept> {
		self.headers.get()
	}
	/// Short-cut to obtain the WebSocketProtocol value
	pub fn protocol(&self) -> Option<&WebSocketProtocol> {
		self.headers.get()
	}
	/// Short-cut to obtain the WebSocketExtensions value
	pub fn extensions(&self) -> Option<&WebSocketExtensions> {
		self.headers.get()
	}
}

impl<R: Reader + Send, W: Writer + Send> WebSocketResponse<R, W, Outbound> {
	/// Create a new outbound WebSocket response.
	pub fn new(status: StatusCode, headers: Headers, version: HttpVersion, reader: R, writer: W) -> WebSocketResponse<R, W, Outbound> {
		WebSocketResponse {
			status: status,
			headers: headers,
			version: version,
			reader: reader,
			writer: writer,
		}
	}
	/// Short-cut to obtain a mutable reference to the WebSocketAccept value
	/// Note that to add a header that does not already exist, ```WebSocketResponse.headers.set()```
	/// must be used.
	pub fn accept_mut(&mut self) -> Option<&mut WebSocketAccept> {
		self.headers.get_mut()
	}
	/// Short-cut to obtain a mutable reference to the WebSocketProtocol value
	/// Note that to add a header that does not already exist, ```WebSocketResponse.headers.set()```
	/// must be used.
	pub fn protocol_mut(&mut self) -> Option<&mut WebSocketProtocol> {
		self.headers.get_mut()
	}
	/// Short-cut to obtain a mutable reference to the WebSocketExtensions value
	/// Note that to add a header that does not already exist, ```WebSocketResponse.headers.set()```
	/// must be used.
	pub fn extensions_mut(&mut self) -> Option<&mut WebSocketExtensions> {
		self.headers.get_mut()
	}
	
	/// Send this response with the given DataFrameSender, DataFrameReceiver and DataFrameConverter
	pub fn send_with<A: DataFrameSender<W>, B: DataFrameReceiver<R>, C: DataFrameConverter<M>, M: WebSocketMessaging>(mut self) -> WebSocketResult<WebSocketClient<A, B, C, R, W, M>> {
		try!(self.write());
		let sender: A = DataFrameSender::new(self.writer);
		let receiver: B = DataFrameReceiver::new(self.reader);
		let converter: C = DataFrameConverter::new();
		Ok(WebSocketClient::new(sender, receiver, converter))
	}
	
	/// Send this response, retrieving the inner Reader and Writer
	pub fn send_into_inner(mut self) -> WebSocketResult<(R, W)> {
		try!(self.write());
		Ok(self.into_inner())
	}
	
	/// Send this response, returning a WebSocketClient ready to transmit/receive data frames
	pub fn send(self) -> WebSocketResult<WebSocketClient<WebSocketSender<W, Remote>, WebSocketReceiver<R, Remote>, WebSocketConverter<WebSocketMessage>, R, W, WebSocketMessage>> {
		self.send_with::<WebSocketSender<W, Remote>, WebSocketReceiver<R, Remote>, WebSocketConverter<WebSocketMessage>, WebSocketMessage>()
	}
	
	fn write(&mut self) -> WebSocketResult<()> {
		try!(write!(&mut self.writer, "{} {}\r\n", self.version, self.status));
		try!(write!(&mut self.writer, "{}\r\n", self.headers));
		Ok(())
	}
}

impl<R: Reader + Send, W: Writer + Send> WebSocketResponse<R, W, Inbound> {
	/// Read an inbound response
	pub fn read(reader: R, writer: W) -> WebSocketResult<WebSocketResponse<R, W, Inbound>> {
		let mut reader = reader;
		let (version, raw_status) = try!(read_status_line(&mut reader));
		let status = match FromPrimitive::from_u16(raw_status.0) {
			Some(status) => { status }
			None => { return Err(WebSocketError::ResponseError("Could not get status code".to_string())); }
		};
		let headers = try!(Headers::from_raw(&mut reader));
		Ok(WebSocketResponse {
			status: status,
			headers: headers,
			version: version,
			reader: reader,
			writer: writer,
		})
	}
	/// Check if this response constitutes a successful handshake
	pub fn validate(&self, key: &WebSocketKey) -> WebSocketResult<()> {
		if self.status != StatusCode::SwitchingProtocols {
			return Err(WebSocketError::ResponseError("Status code must be Switching Protocols".to_string()));
		}
		if self.accept() != Some(&(WebSocketAccept::new(key))) {
			return Err(WebSocketError::ResponseError("Sec-WebSocket-Accept is invalid".to_string()));
		}
		if self.headers.get() != Some(&(Upgrade(vec![Protocol::WebSocket]))) {
			return Err(WebSocketError::ResponseError("Upgrade field must be WebSocket".to_string()));
		}
		if self.headers.get() != Some(&(Connection(vec![ConnectionOption::ConnectionHeader("Upgrade".to_string())]))) {
			return Err(WebSocketError::ResponseError("Connection field must be 'Upgrade'".to_string()));
		}
		Ok(())
	}
	
	/// Consume this response and return a WebSocketClient ready to transmit/receive data frames
	/// using the specified DataFrameSender, DataFrameReceiver and DataFrameConverter.
	///
	/// Does not check if the response was valid. Use ```validate()``` to ensure that the response constitutes a successful handshake.
	pub fn begin_with<A: DataFrameSender<W>, B: DataFrameReceiver<R>, C: DataFrameConverter<M>, M: WebSocketMessaging>(self) -> WebSocketClient<A, B, C, R, W, M> {
		let sender: A = DataFrameSender::new(self.writer);
		let receiver: B = DataFrameReceiver::new(self.reader);
		let converter: C = DataFrameConverter::<M>::new();
		WebSocketClient::new(sender, receiver, converter)
	}
	/// Consume this response and return a WebSocketClient ready to transmit/receive data frames
	pub fn begin(self) -> WebSocketClient<WebSocketSender<W, Local>, WebSocketReceiver<R, Local>, WebSocketConverter<WebSocketMessage>, R, W, WebSocketMessage> {
		self.begin_with::<WebSocketSender<W, Local>, WebSocketReceiver<R, Local>, WebSocketConverter<WebSocketMessage>, WebSocketMessage>()
	}
}