#![stable]
//! Structs for dealing with WebSocket requests and responses.

pub use self::request::WebSocketRequest;
pub use self::response::WebSocketResponse;

pub mod request;
pub mod response;