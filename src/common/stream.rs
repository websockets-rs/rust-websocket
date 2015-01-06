#![stable]
//! Provides a default implementation of a stream which can be used with Rust-WebSocket.
//! 
//! A single type is provided, capable of carrying either a TcpStream or an SslStream<TcpStream>.
//! This is used with convenience methods such as ```WebSocketRequest::connect()```, ```WebSocketServer::bind()```
//! and ```WebSocketServer::bind_secure()```.

use std::io::net::tcp::TcpStream;
use std::io::IoResult;
use openssl::ssl::SslStream;

/// Represents a stream capable of carrying a WebSocket connection.
///
/// Can either be a normal TcpStream or an SSL-backed TcpStream.
/// This is the default stream used within Rust-WebSocket, however any Reader or Writer can be used
/// if desired, using the un-typed functions.
#[derive(Clone)]
#[stable]
pub enum WebSocketStream {
	/// A normal (non-secure) stream
	Normal(TcpStream),
	/// A secure stream
	Secure(SslStream<TcpStream>),
	/// A normal (non-secure) stream which carries a peaked at byte
	BufferNormal(TcpStream, u8),
	/// A secure stream which carries a peaked at byte
	BufferSecure(SslStream<TcpStream>, u8),
}

/// A trait that allows for checking if data is available on the stream
pub trait DataAvailable: Send {
	/// True if data is available
	fn data_available(&mut self) -> bool;
}

impl Reader for WebSocketStream {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<uint> {
		let mut to_normal = None;
		let mut to_secure = None;
        let output = match *self {
        	WebSocketStream::Normal(ref mut inner) => inner.read(buf),
			WebSocketStream::Secure(ref mut inner) => inner.read(buf),
			WebSocketStream::BufferNormal(ref mut inner, byte) => {
				to_normal = Some(inner.clone());
				buf[0] = byte;
				Ok(
					if buf.len() > 1 {
						let mut buf = buf.slice_from_mut(1);
						let output = try!(inner.read(buf));
						output + 1
					}
					else { 1 }
				)
			}
			WebSocketStream::BufferSecure(ref mut inner, byte) => {
				to_secure = Some(inner.clone());
				buf[0] = byte;
				Ok(
					if buf.len() > 1 {
						let mut buf = buf.slice_from_mut(1);
						let output = try!(inner.read(buf));
						output + 1
					}
					else { 1 }
				)
			}
		};
		match to_normal { 
			Some(inner) => { *self = WebSocketStream::Normal(inner); }
			None => { }
		}
		match to_secure { 
			Some(inner) => { *self = WebSocketStream::Secure(inner); }
			None => { }
		}
		output
	}
}

impl Writer for WebSocketStream {
	fn write(&mut self, msg: &[u8]) -> IoResult<()> {
		match *self {
			WebSocketStream::Normal(ref mut inner) => inner.write(msg),
			WebSocketStream::Secure(ref mut inner) => inner.write(msg),
			WebSocketStream::BufferNormal(ref mut inner, _) => inner.write(msg),
			WebSocketStream::BufferSecure(ref mut inner, _) => inner.write(msg),
		}
	}
}

impl DataAvailable for WebSocketStream {
	/// True if data is available
	fn data_available(&mut self) -> bool {
		let mut to_nbuffer = None;
		let mut to_sbuffer = None;
		match *self {
        	WebSocketStream::Normal(ref mut inner) => {
				inner.set_read_timeout(Some(5));
				let read = inner.read_byte();
				inner.set_read_timeout(None);
				match read {
					Ok(byte) => to_nbuffer = Some((inner.clone(), byte)),
					Err(_) => { }
				}
			}
			WebSocketStream::Secure(ref mut inner) => {
				let read = {
					let mut stream = inner.get_mut();
					stream.set_read_timeout(Some(5));
					let save = stream.read_byte();
					stream.set_read_timeout(None);
					save
				};
				match read {
					Ok(byte) => to_sbuffer = Some((inner.clone(), byte)),
					Err(_) => { }
				}
			}
			_ => { return true; }
		}
		match to_nbuffer { 
			Some((inner, byte)) => { *self = WebSocketStream::BufferNormal(inner, byte); return true; }
			None => { }
		}
		match to_sbuffer { 
			Some((inner, byte)) => { *self = WebSocketStream::BufferSecure(inner, byte); return true; }
			None => { }
		}
		false
	}
}