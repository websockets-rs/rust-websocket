use super::client::{WebSocketClient, serverside_client};
use std::io::net::tcp::{TcpListener, TcpAcceptor};
use std::io::net::ip::ToSocketAddr;
use std::io::{Listener, Acceptor};
use std::io::IoResult;

pub struct WebSocketServer {
	listener: TcpListener,
}

impl WebSocketServer {
	pub fn bind<A: ToSocketAddr>(addr: A) -> IoResult<WebSocketServer> {
		let listener = try!(TcpListener::bind(addr));
		Ok(WebSocketServer {
			listener: listener,
		})
	}
}

impl Listener<WebSocketClient, WebSocketAcceptor> for WebSocketServer {
	fn listen(self) -> IoResult<WebSocketAcceptor> {
		let acceptor = try!(self.listener.listen());
		Ok(WebSocketAcceptor {
			acceptor: acceptor,
		})
	}
}

pub struct WebSocketAcceptor {
	acceptor: TcpAcceptor,
}

impl Acceptor<WebSocketClient> for WebSocketAcceptor {
	fn accept(&mut self) -> IoResult<WebSocketClient> {
		let stream = try!(self.acceptor.accept());
		serverside_client(stream)
	}
} 