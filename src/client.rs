use ws::{Sender, Receiver, Message};
use ws::receiver::{DataFrameIterator, MessageIterator};
use result::WebSocketResult;

pub struct WebSocketClient<D, S, R> {
	sender: S,
	receiver: R
}

impl<D, S: Sender<D>, R: Receiver<D>> WebSocketClient<D, S, R> {
	/// Creates a WebSocketClient from the given Sender and Receiver.
	///
	/// Essentially the opposite of `WebSocketClient.split()`.
	pub fn new(sender: S, receiver: R) -> WebSocketClient<D, S, R> {
		WebSocketClient {
			sender: sender,
			receiver: receiver
		}
	}
	/// Sends a single data frame to the remote endpoint.
	pub fn send_dataframe(&mut self, dataframe: D) -> WebSocketResult<()> {
		self.sender.send_dataframe(dataframe)
	}
	/// Sends a single message to the remote endpoint.
	pub fn send_message<M>(&mut self, message: M) -> WebSocketResult<()> 
		where M: Message<D>, <M as Message<D>>::DataFrameIterator: Iterator<Item = D> {
		
		self.sender.send_message(message)
	}
	/// Reads a single data frame from the remote endpoint.
	pub fn recv_dataframe(&mut self) -> WebSocketResult<D> {
		self.receiver.recv_dataframe()
	}
	/// Returns an iterator over incoming data frames.
	pub fn incoming_dataframes<'a>(&'a mut self) -> DataFrameIterator<'a, R, D> {
		self.receiver.incoming_dataframes()
	}
	/// Reads a single message from this receiver.
	pub fn recv_message(&mut self) -> WebSocketResult<<R as Receiver<D>>::Message> {
		self.receiver.recv_message()
	}
	/// Returns an iterator over incoming messages.
	pub fn incoming_messages<'a>(&'a mut self) -> MessageIterator<'a, R, D> {
		self.receiver.incoming_messages()
	}
	/// Split this client into its constituent Sender and Receiver pair.
	///
	/// This allows the Sender and Receiver to be sent to different threads.
	pub fn split(self) -> (S, R) {
		(self.sender, self.receiver)
	}
}