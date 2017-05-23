use std::io;
use std::net::SocketAddr;
use server::{WsServer, NoTlsAcceptor};
use tokio_core::net::TcpListener;
pub use tokio_core::reactor::Handle;

#[cfg(any(feature="async-ssl"))]
use native_tls::{TlsStream, TlsAcceptor};

pub type Server<S> = WsServer<S, TcpListener>;

impl WsServer<NoTlsAcceptor, TcpListener> {
	/// Bind this Server to this socket
	pub fn bind(addr: &SocketAddr, handle: &Handle) -> io::Result<Self> {
		Ok(Server {
		       listener: TcpListener::bind(addr, handle)?,
		       ssl_acceptor: NoTlsAcceptor,
		   })
	}

	/// Wait for and accept an incoming WebSocket connection, returning a WebSocketRequest
	pub fn incoming(&mut self) {
		unimplemented!();
	}
}

#[cfg(any(feature="async-ssl"))]
impl WsServer<TlsAcceptor, TcpListener> {
	/// Bind this Server to this socket
	pub fn bind_secure(
		addr: &SocketAddr,
		acceptor: TlsAcceptor,
		handle: &Handle,
	) -> io::Result<Self> {
		Ok(Server {
		       listener: TcpListener::bind(addr, handle)?,
		       ssl_acceptor: acceptor,
		   })
	}

	/// Wait for and accept an incoming WebSocket connection, returning a WebSocketRequest
	pub fn incoming(&mut self) {
		unimplemented!();
	}
}
