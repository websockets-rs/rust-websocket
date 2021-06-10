//! This is part of `websocket` crate that is independent from `hyper`.
//! It contains code for processing WebSocket streams,
//! (after HTTP upgrade already happened)
//! WebSocket message definition, some error type.
//!
//! For now it still optionally depends on `futures 0.1` and `tokio-io 0.1`, but that's going to
//! be extracted to a separate crate, if `rust-websocket`'s depelopment continues.
//!
//! Note that there is no normal example of usage so far. See usual `websocket` crate or
//! alternative like `tungstenite`.

#![allow(clippy::match_ref_pats, clippy::needless_doctest_main)]
extern crate byteorder;

extern crate rand;
#[macro_use]
extern crate bitflags;



pub mod dataframe;
pub mod header;
pub mod message;
pub mod result;
pub mod stream;
pub mod ws;

pub use crate::message::Message;
pub use crate::message::OwnedMessage;
