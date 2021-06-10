//! Build and use synchronous websocket clients.
//!
//! In general pick a style you would like to write in and use `ClientBuilder`
//! to create your websocket connections. Use the `.async_connect` functions to create
//! async connections, and the normal `.connect` functions for synchronous clients.
//! The `ClientBuilder` creates both async and sync connections, the actual sync and
//! async clients live in the `client::sync` and `client::async` modules, respectively.
//!
//! Many of the useful things from this module will be hoisted and re-exported under the
//! `websocket::{sync, async}::client` module which will have all sync or all async things.

pub mod builder;
pub use self::builder::{ClientBuilder, ParseError, Url};


pub mod sync;
