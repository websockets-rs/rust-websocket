//! A module containing the traits and structs that lower layer of Rust-WebSocket is based on.
//!
//! This should not need to be used by regular users.
//!
//! Rust-WebSocket is based on three core traits: `Message`, `Sender` and `Receiver`. These
//! traits have default implementations outside this module, however can be implemented
//! by a user to extend the functionality provided.
//!
//! If a user wishes to use a different representation of a data frame, all three traits
//! must be implemented by the user. If a user wishes to use a different representation
//! of a message (but the same data frame), they must implement the `Message` and `Receiver`
//! traits.
//!
//! A WebSocket message type must implement `Message<D>` where `D` is the type of data frame
//! that the message can be converted to/from.
//!
//! When sending a message, the message is converted into an iterator with its `into_iter()`
//! method, which allows the message to output data frames forming a fragmented message
//! where each data frame is sent immediately to be reassembled at the remote endpoint.
//!
//! The type of data frame can be any type, however, if you choose a data frame type other than
//! `DataFrame`, you will also have to implement the `Sender` and `Receiver` traits to
//! send and receive data frames.
//!
//! A `Sender<D>` sends a data frame of type `D`, typically wrapping an underlying Writer,
//! by implementing the `send_dataframe()` method. The `send_message()` method has a default
//! implementation which turns the message into an iterator and then continually calls
//! `send_dataframe()` with the frames from the iterator.
//!
//! To make life easier for a `Sender`, several utility functions are provided which write
//! various pieces of data to a Writer. These are found within the `util` module.
//!
//! A Receiver<D> receives data frames of type D and messages of type Receiver::Message,
//! typically wrapping an underlying Reader, by implementing the `recv_dataframe()` and
//! `recv_message_dataframes()` methods. The `recv_message_dataframes()` method has to
//! form a `Vec` of data frames which comprise one whole, single message.
//!
//! To make life easier for a `Receiver`, several utility functions are provided which read
//! various pieces of data from a Reader. These are found within the `util` module.
pub use self::message::Message;

#[cfg(feature = "sync")]
pub use self::receiver::Receiver;
#[cfg(feature = "sync")]
pub use self::receiver::{DataFrameIterator, MessageIterator};
#[cfg(feature = "sync")]
pub use self::sender::Sender;

pub mod dataframe;
pub mod message;
pub mod util;

#[cfg(feature = "sync")]
pub mod receiver;
#[cfg(feature = "sync")]
pub mod sender;
