//! Utility functions for various portions of Rust-WebSocket.

pub mod header;
pub mod mask;

/// Represents a local endpoint
#[derive(Show, Copy)]
#[stable]
pub struct Local;

/// Represents a remote endpoint
#[derive(Show, Copy)]
#[stable]
pub struct Remote;

/// Represents an inbound object
#[derive(Show, Copy)]
#[stable]
pub struct Inbound;

/// Represents an outbound object
#[derive(Show, Copy)]
#[stable]
pub struct Outbound;