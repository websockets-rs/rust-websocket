use super::util::header::HeaderCollection;
use std::ascii::AsciiExt;

pub trait CheckWebSocketHeader {
	fn check_request(&self) -> bool;
	fn check_response(&self) -> bool;
}

impl CheckWebSocketHeader for HeaderCollection {
	fn check_request(&self) -> bool {
		self.contains_field("Host")
		&& self.contains_field("Connection")
		&& self.contains_field("Upgrade")
		&& self.contains_field("Sec-WebSocket-Version")
		&& self.contains_field("Sec-WebSocket-Key")
		&& self.get("Sec-WebSocket-Version").unwrap().as_slice() == "13"
		&& self.get("Upgrade").unwrap().as_slice().to_ascii_lower().contains("websocket")
		&& self.get("Connection").unwrap().as_slice().to_ascii_lower().contains("upgrade")
		
	}
	
	fn check_response(&self) -> bool {
		self.contains_field("Upgrade")
		&& self.contains_field("Sec-WebSocket-Accept")
		&& self.get("Upgrade").unwrap().as_slice().eq_ignore_ascii_case("websocket")
	}
}