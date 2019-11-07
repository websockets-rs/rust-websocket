//! Useful `Codec` types for asynchronously encoding and decoding messages.
//!
//! This is intended to be used with the `Framed` type in the `tokio-io` crate.
//! This module contains an `http` codec which can be used to send and receive
//! hyper HTTP requests and responses asynchronously.
//! See it's module level documentation for more info.
//!
//! Second but most importantly this module contains a codec for asynchronously
//! encoding and decoding websocket messages (and dataframes if you want to go
//! more low level) in the `ws` module.
//! See it's module level documentation for more info.

pub mod http;
pub use websocket_base::codec::ws;
