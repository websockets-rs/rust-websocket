//! Module containing default implementations for various portions of Rust-WebSocket.
//!
//! These are the quickest way to use Rust-WebSocket, and represent the most common
//! features required by a WebSocket application.

pub use self::dataframe::{WebSocketDataFrame, WebSocketOpcode};
pub use self::message::WebSocketMessage;
pub use self::sender::WebSocketSender;
pub use self::receiver::WebSocketReceiver;
pub use self::stream::WebSocketStream;

pub use ws::{Message, Sender, Receiver};
pub use ws::{DataFrameIterator, MessageIterator};

pub mod dataframe;
pub mod message;
pub mod sender;
pub mod receiver;
pub mod stream;

/// The default WebSocket client type.
pub type WebSocketClient<R, W> = super::Client<WebSocketDataFrame, WebSocketSender<W>, WebSocketReceiver<R>>;

/// Represents an inbound object
#[derive(Show, Copy)]
#[stable]
pub struct Inbound;

/// Represents an outbound object
#[derive(Show, Copy)]
#[stable]
pub struct Outbound;