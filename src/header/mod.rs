//! Structs representing headers relevant in a WebSocket context.
//!
//! These headers are commonly used in WebSocket requests and responses.
//! The `Header` trait from the `hyper` crate is used.

pub use self::key::WebSocketKey;
pub use self::accept::WebSocketAccept;
pub use self::protocol::WebSocketProtocol;
pub use self::version::WebSocketVersion;
pub use self::extensions::WebSocketExtensions;
pub use self::origin::Origin;
pub use hyper::header::*;

mod accept;
mod key;
mod protocol;
mod version;
pub mod extensions;
mod origin;
