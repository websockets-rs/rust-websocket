use message::Message;
use result::WebSocketResult;

pub trait Sender {
	type Message: Message;
	
	fn send_dataframe(&mut self, dataframe: <Self::Message as Message>::DataFrame) -> WebSocketResult<()>;
	
	fn send_message(&mut self, mut message: Self::Message) -> WebSocketResult<()> {
		for dataframe in *message.into_iter() {
			try!(self.send_dataframe(dataframe));
		}
		Ok(())
	}
}