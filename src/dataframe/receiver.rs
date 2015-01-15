use std::iter::Unfold;
use message::Message;
use result::WebSocketResult;

pub trait Receiver {
	type Message: Message;
	
	fn recv_dataframe(&mut self) -> WebSocketResult<<Self::Message as Message>::DataFrame>;
	
	fn recv_message(&mut self) -> WebSocketResult<Self::Message> {
		let iter = Unfold::new((), |_| Some(self.recv_dataframe()));
		Message::from_iter(iter)
	}
}
