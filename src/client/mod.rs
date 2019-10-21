//! Build and use asynchronously or synchronous websocket clients.
//!
//! This crate is split up into a synchronous and asynchronous half.
//! These can be turned on and off by switching the `sync` and `async` features
//! on and off (plus `sync-ssl` and `async-ssl` for SSL connections).
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

#[cfg(feature = "async")]
pub mod r#async;

#[cfg(feature = "sync")]
pub mod sync;
