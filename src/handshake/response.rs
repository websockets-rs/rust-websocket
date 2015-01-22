#![unstable]
//! Structs for WebSocket responses
use std::option::Option;
use std::num::FromPrimitive;

use hyper::status::StatusCode;
use hyper::version::HttpVersion;
use hyper::header::Headers;
use hyper::header::common::{Connection, Upgrade};
use hyper::header::connection::ConnectionOption;
use hyper::header::common::upgrade::Protocol;
use hyper::http::read_status_line;

use header::{WebSocketKey, WebSocketAccept, WebSocketProtocol, WebSocketExtensions};

use client::Client;
use common::{Inbound, Outbound};
use common::{WebSocketClient, WebSocketStream};
use common::{WebSocketSender, WebSocketReceiver};
use result::{WebSocketResult, WebSocketError};
use ws::{Sender, Receiver};

/// The most common inbound response type, provided for convenience.
pub type WebSocketInboundResponse = WebSocketResponse<WebSocketStream, WebSocketStream, Inbound>;
/// The most common outbound response type, provided for convenience.
pub type WebSocketOutboundResponse = WebSocketResponse<WebSocketStream, WebSocketStream, Outbound>;


/// Represents a WebSocket response
/// 
/// A WebSocketResponse can sent by a WebSocket server by calling ```accept()``` on the WebSocketRequest 
/// and then ```send()``` on the response, or received by a WebSocket client to be consumed into a
/// Client with ```begin()```.
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
		/// Returns a reference to the inner Reader.
	pub fn get_reader(&self) -> &R {
		&self.reader
	}
	/// Returns a reference to the inner Writer.
	pub fn get_writer(&self) -> &W {
		&self.writer
	}
	/// Returns a mutable reference to the inner Reader.
	pub fn get_mut_reader(&mut self) -> &mut R {
		&mut self.reader
	}
	/// Returns a mutable reference to the inner Writer.
	pub fn get_mut_writer(&mut self) -> &mut W {
		&mut self.writer
	}
	/// Return the inner Reader and Writer
	pub fn into_inner(self) -> (R, W) {
		(self.reader, self.writer)
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
	
	/// Send this response with the given data frame type D, Sender B and Receiver C.
	pub fn send_with<D, B, C>(mut self, sender: B, receiver: C) -> WebSocketResult<Client<D, B, C>> 
		where B: Sender<D>, C: Receiver<D> {
		
		try!(write!(&mut self.writer, "{} {}\r\n", self.version, self.status));
		try!(write!(&mut self.writer, "{}\r\n", self.headers));
		Ok(Client::new(sender, receiver))
	 }
	
	/// Send this response, retrieving the inner Reader and Writer
	pub fn send_into_inner(mut self) -> WebSocketResult<(R, W)> {
		try!(write!(&mut self.writer, "{} {}\r\n", self.version, self.status));
		try!(write!(&mut self.writer, "{}\r\n", self.headers));
		Ok(self.into_inner())
	}
	
	/// Send this response, returning a Client ready to transmit/receive data frames
	pub fn send(mut self) -> WebSocketResult<WebSocketClient<R, W>> {
		try!(write!(&mut self.writer, "{} {}\r\n", self.version, self.status));
		try!(write!(&mut self.writer, "{}\r\n", self.headers));
		let sender = WebSocketSender::new(self.writer, false);
		let receiver = WebSocketReceiver::new(self.reader, false);
		Ok(Client::new(sender, receiver))
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
	
	/// Consume this response and return a Client ready to transmit/receive data frames
	/// using the data frame type D, Sender B and Receiver C.
	///
	/// Does not check if the response was valid. Use ```validate()``` to ensure that the response constitutes a successful handshake.
	pub fn begin_with<D, B, C>(self, sender: B, receiver: C) -> Client<D, B, C> 
		where B: Sender<D>, C: Receiver<D> {
		Client::new(sender, receiver)
	}
	/// Consume this response and return a Client ready to transmit/receive data frames.
	///
	/// Does not check if the response was valid. Use ```validate()``` to ensure that the response constitutes a successful handshake.
	pub fn begin(self) -> WebSocketClient<R, W> {
		let sender = WebSocketSender::new(self.writer, true);
		let receiver = WebSocketReceiver::new(self.reader, true);
		Client::new(sender, receiver)
	}
}