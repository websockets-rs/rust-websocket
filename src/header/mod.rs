//! Structs representing headers relevant in a WebSocket context.
//!
//! These headers are commonly used in WebSocket requests and responses.
//! The `Header` trait from the `hyper` crate is used.

pub use self::accept::WebSocketAccept;
pub use self::extensions::WebSocketExtensions;
pub use self::key::WebSocketKey;
pub use self::origin::Origin;
pub use self::protocol::WebSocketProtocol;
pub use self::version::WebSocketVersion;
pub use hyper::header::*;

mod accept;
pub mod extensions;
mod key;
mod origin;
mod protocol;
mod version;
