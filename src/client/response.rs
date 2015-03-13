//! Structs for WebSocket responses
use std::option::Option;
use std::num::FromPrimitive;
use std::io::{Read, Write};

use hyper::status::StatusCode;
use hyper::version::HttpVersion;
use hyper::header::Headers;
use hyper::header::{Connection, ConnectionOption};
use hyper::header::{Upgrade, Protocol};
use hyper::http::read_status_line;

use unicase::UniCase;

use header::{WebSocketAccept, WebSocketProtocol, WebSocketExtensions};

use client::{Client, Request, Sender, Receiver};
use result::{WebSocketResult, WebSocketError};
use dataframe::DataFrame;
use ws;

/// Represents a WebSocket response.
#[derive(Debug)]
pub struct Response<R: Read, W: Write> {
	/// The status of the response
	pub status: StatusCode,
	/// The headers contained in this response
	pub headers: Headers,
	/// The HTTP version of this response
	pub version: HttpVersion,
	
	request: Request<R, W>
}

unsafe impl<R, W> Send for Response<R, W> where R: Read + Send, W: Write + Send { }

impl<R: Read, W: Write> Response<R, W> {
	/// Reads a Response off the stream associated with a Request.
	///
	/// This is called by Request.send(), and does not need to be called by the user.
	pub fn read(mut request: Request<R, W>) -> WebSocketResult<Response<R, W>> {
		let (status, version, headers) = {
			let reader = request.get_mut_reader();
			let (version, raw_status) = try!(read_status_line(reader));
			let status = match FromPrimitive::from_u16(raw_status.0) {
				Some(status) => { status }
				None => { return Err(WebSocketError::ResponseError("Could not get status code".to_string())); }
			};
			let headers = try!(Headers::from_raw(reader));
			(status, version, headers)
		};
		
		Ok(Response {
			status: status,
			headers: headers,
			version: version,
			request: request
		})
	}
	
	/// Short-cut to obtain the WebSocketAccept value.
	pub fn accept(&self) -> Option<&WebSocketAccept> {
		self.headers.get()
	}
	/// Short-cut to obtain the WebSocketProtocol value.
	pub fn protocol(&self) -> Option<&WebSocketProtocol> {
		self.headers.get()
	}
	/// Short-cut to obtain the WebSocketExtensions value.
	pub fn extensions(&self) -> Option<&WebSocketExtensions> {
		self.headers.get()
	}
		/// Returns a reference to the inner Reader.
	pub fn get_reader(&self) -> &R {
		self.request.get_reader()
	}
	/// Returns a reference to the inner Writer.
	pub fn get_writer(&self) -> &W {
		self.request.get_writer()
	}
	/// Returns a mutable reference to the inner Reader.
	pub fn get_mut_reader(&mut self) -> &mut R {
		self.request.get_mut_reader()
	}
	/// Returns a mutable reference to the inner Writer.
	pub fn get_mut_writer(&mut self) -> &mut W {
		self.request.get_mut_writer()
	}
	/// Returns a reference to the request associated with this response.
	pub fn get_request(&self) -> &Request<R, W> {
		&self.request
	}
	/// Return the inner Reader and Writer.
	pub fn into_inner(self) -> (R, W) {
		self.request.into_inner()
	}
	
	/// Check if this response constitutes a successful handshake.
	pub fn validate(&self) -> WebSocketResult<()> {
		if self.status != StatusCode::SwitchingProtocols {
			return Err(WebSocketError::ResponseError("Status code must be Switching Protocols".to_string()));
		}
		let key = try!(self.request.key().ok_or(
			WebSocketError::RequestError("Request Sec-WebSocket-Key was invalid".to_string())
		));
		if self.accept() != Some(&(WebSocketAccept::new(key))) {
			return Err(WebSocketError::ResponseError("Sec-WebSocket-Accept is invalid".to_string()));
		}
		if self.headers.get() != Some(&(Upgrade(vec![Protocol::WebSocket]))) {
			return Err(WebSocketError::ResponseError("Upgrade field must be WebSocket".to_string()));
		}
		if self.headers.get() != Some(&(Connection(vec![ConnectionOption::ConnectionHeader(UniCase("Upgrade".to_string()))]))) {
			return Err(WebSocketError::ResponseError("Connection field must be 'Upgrade'".to_string()));
		}
		Ok(())
	}
	
	/// Consume this response and return a Client ready to transmit/receive data frames
	/// using the data frame type D, Sender B and Receiver C.
	///
	/// Does not check if the response was valid. Use `validate()` to ensure that the response constitutes a successful handshake.
	pub fn begin_with<D, B, C>(self, sender: B, receiver: C) -> Client<D, B, C> 
		where B: ws::Sender<D>, C: ws::Receiver<D> {
		Client::new(sender, receiver)
	}
	/// Consume this response and return a Client ready to transmit/receive data frames.
	///
	/// Does not check if the response was valid. Use `validate()` to ensure that the response constitutes a successful handshake.
	pub fn begin(self) -> Client<DataFrame, Sender<W>, Receiver<R>> {
		let (reader, writer) = self.into_inner();
		let sender = Sender::new(writer);
		let receiver = Receiver::new(reader);
		Client::new(sender, receiver)
	}
}
