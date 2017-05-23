//! Provides an implementation of a WebSocket server
#[cfg(feature="ssl")]
use native_tls::TlsAcceptor;
pub use self::upgrade::{Request, HyperIntoWsError};

pub mod upgrade;

#[cfg(feature="async")]
pub mod async;

pub mod sync;

/// Marker struct for a struct not being secure
#[derive(Clone)]
pub struct NoTlsAcceptor;
/// Trait that is implemented over NoSslAcceptor and SslAcceptor that
/// serves as a generic bound to make a struct with.
/// Used in the Server to specify impls based on wether the server
/// is running over SSL or not.
pub trait OptionalTlsAcceptor {}
impl OptionalTlsAcceptor for NoTlsAcceptor {}
#[cfg(feature="ssl")]
impl OptionalTlsAcceptor for TlsAcceptor {}

pub struct WsServer<S, L>
	where S: OptionalTlsAcceptor
{
	listener: L,
	ssl_acceptor: S,
}

