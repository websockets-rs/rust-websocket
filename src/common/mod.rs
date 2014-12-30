//! Contains some common types for use with Rust-WebSocket
#![unstable]

pub use self::error::WebSocketError;
pub use self::stream::WebSocketStream;

pub mod stream;
pub mod error;

/// The type used for WebSocket results
#[stable]
pub type WebSocketResult<T> = Result<T, WebSocketError>;

/// Represents a local endpoint
#[deriving(Clone, Show, Copy)]
#[stable]
pub struct Local;

/// Represents a remote endpoint
#[deriving(Clone, Show, Copy)]
#[stable]
pub struct Remote;

/// Represents an inbound object
#[deriving(Clone, Show, Copy)]
#[stable]
pub struct Inbound;

/// Represents an outbound object
#[deriving(Clone, Show, Copy)]
#[stable]
pub struct Outbound;