extern crate hyper;
extern crate openssl;

use openssl::ssl::SslStream;
use hyper::client::pool::PooledStream;
use std::net::TcpStream;
use std::io::{
	Read,
	Write,
	self,
};
use stream::{
	Stream,
	AsTcpStream,
};
use hyper::net::{
	NetworkStream,
	HttpStream,
	HttpsStream,
};

impl<S> Stream for S
where S: NetworkStream,
{
	type R = Self;
	type W = Self;

	fn reader(&mut self) -> &mut Read {
		self
	}

	fn writer(&mut self) -> &mut Read {
		self
	}

	fn split(self) -> io::Result<(Self::R, Self::W)> {
		if let Some(http) = self.downcast_ref::<HttpStream>() {
			Ok((http.clone(), self))
		} else {
			Err(io::Error::new(
				io::ErrorKind::Other,
				"Unknown implementation of NetworkStream found!",
			))
		}
	}
}

impl<S> AsTcpStream for S
where S: NetworkStream,
{
	fn as_tcp(&self) -> &TcpStream {
		unimplemented!();
	}
}
