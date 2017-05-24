pub use tokio_core::reactor::Handle;
pub use tokio_io::codec::Framed;
pub use tokio_core::net::TcpStream;
pub use futures::Future;
use hyper::header::Headers;

use result::WebSocketError;
use codec::ws::MessageCodec;
use message::OwnedMessage;

#[cfg(feature="async-ssl")]
pub use tokio_tls::TlsStream;

pub type Client<S> = Framed<S, MessageCodec<OwnedMessage>>;

pub type ClientNew<S> = Box<Future<Item = (Client<S>, Headers), Error = WebSocketError>>;
