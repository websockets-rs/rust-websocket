//! Structs representing headers relevant in a WebSocket context

pub use self::key::WebSocketKey;
pub use self::accept::WebSocketAccept;
pub use self::protocol::WebSocketProtocol;
pub use self::version::WebSocketVersion;
pub use self::extensions::WebSocketExtensions;
pub use self::origin::Origin;
pub use hyper::header::Headers;

mod accept;
mod key;
mod protocol;
mod version;
mod extensions;
mod origin;