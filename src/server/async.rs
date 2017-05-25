use std::io;
use std::net::ToSocketAddrs;
use std::net::SocketAddr;
use server::{WsServer, NoTlsAcceptor};
use tokio_core::net::{TcpListener, TcpStream};
use futures::{Stream, Future};
use server::upgrade::async::{IntoWs, Upgrade};
use server::InvalidConnection;
use bytes::BytesMut;
pub use tokio_core::reactor::Handle;

#[cfg(any(feature="async-ssl"))]
use native_tls::TlsAcceptor;
#[cfg(any(feature="async-ssl"))]
use tokio_tls::{TlsAcceptorExt, TlsStream};

pub type Server<S> = WsServer<S, TcpListener>;

pub type Incoming<S> = Box<Stream<Item = (Upgrade<S>, SocketAddr),
                                  Error = InvalidConnection<S, BytesMut>>>;

pub enum AcceptError<E> {
	Io(io::Error),
	Upgrade(E),
}

impl WsServer<NoTlsAcceptor, TcpListener> {
	pub fn bind<A: ToSocketAddrs>(addr: A, handle: &Handle) -> io::Result<Self> {
		let tcp = ::std::net::TcpListener::bind(addr)?;
		let address = tcp.local_addr()?;
		Ok(Server {
		       listener: TcpListener::from_listener(tcp, &address, handle)?,
		       ssl_acceptor: NoTlsAcceptor,
		   })
	}

	pub fn incoming(self) -> Incoming<TcpStream> {
		let future = self.listener
		                 .incoming()
		                 .map_err(|e| {
			                          InvalidConnection {
			                              stream: None,
			                              parsed: None,
			                              buffer: None,
			                              error: e.into(),
			                          }
			                         })
		                 .and_then(|(stream, a)| {
			stream.into_ws()
			      .map_err(|(stream, req, buf, err)| {
				               InvalidConnection {
				                   stream: Some(stream),
				                   parsed: req,
				                   buffer: Some(buf),
				                   error: err,
				               }
				              })
			      .map(move |u| (u, a))
		});
		Box::new(future)
	}
}

#[cfg(any(feature="async-ssl"))]
impl WsServer<TlsAcceptor, TcpListener> {
	pub fn bind_secure<A: ToSocketAddrs>(
		addr: A,
		acceptor: TlsAcceptor,
		handle: &Handle,
	) -> io::Result<Self> {
		let tcp = ::std::net::TcpListener::bind(addr)?;
		let address = tcp.local_addr()?;
		Ok(Server {
		       listener: TcpListener::from_listener(tcp, &address, handle)?,
		       ssl_acceptor: acceptor,
		   })
	}

	pub fn incoming(self) -> Incoming<TlsStream<TcpStream>> {
		let acceptor = self.ssl_acceptor;
		let future = self.listener
		                 .incoming()
		                 .map_err(|e| {
			                          InvalidConnection {
			                              stream: None,
			                              parsed: None,
			                              buffer: None,
			                              error: e.into(),
			                          }
			                         })
		                 .and_then(move |(stream, a)| {
			acceptor.accept_async(stream)
			        .map_err(|e| {
				InvalidConnection {
					stream: None,
					parsed: None,
					buffer: None,
					// TODO: better error types
					error: io::Error::new(io::ErrorKind::Other, e).into(),
				}
			})
			        .map(move |s| (s, a))
		})
		                 .and_then(|(stream, a)| {
			stream.into_ws()
			      .map_err(|(stream, req, buf, err)| {
				               InvalidConnection {
				                   stream: Some(stream),
				                   parsed: req,
				                   buffer: Some(buf),
				                   error: err,
				               }
				              })
			      .map(move |u| (u, a))
		});
		Box::new(future)
	}
}
