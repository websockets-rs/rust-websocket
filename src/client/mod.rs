pub mod builder;
pub use self::builder::{ClientBuilder, Url, ParseError};

#[cfg(feature="async")]
pub mod async;

#[cfg(feature="sync")]
pub mod sync;
