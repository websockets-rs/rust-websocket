extern crate websocket;
extern crate rustc_serialize as serialize;

use websocket::client::request::Url;
use websocket::{Client, Message, Sender, Receiver};
use websocket::dataframe::Opcode;
use serialize::json;

fn main() {
	let addr = "ws://127.0.0.1:9001".to_string();
	let agent = "rust-websocket";

	println!("Using fuzzingserver {}", addr);
	println!("Using agent {}", agent);

	println!("Running test suite...");

	let mut current_case_id = 1;
	let case_count = get_case_count(addr.clone());

	while current_case_id <= case_count {
		let url = addr.clone() + "/runCase?case=" + &current_case_id.to_string()[..] + "&agent=" + agent;

		let ws_uri = Url::parse(&url[..]).unwrap();
		let request = Client::connect(ws_uri).unwrap();
		let response = request.send().unwrap();
		match response.validate() {
			Ok(()) => (),
			Err(e) => {
				println!("{:?}", e);
				current_case_id += 1;
				continue;
			}
		}
		let (mut sender, mut receiver) = response.begin().split();

		println!("Executing test case: {}/{}", current_case_id, case_count);

		for message in receiver.incoming_messages() {
			let message = match message {
				Ok(message) => message,
				Err(e) => {
					println!("Error: {:?}", e);
					let _ = sender.send_message(&Message::close());
					break;
				}
			};

			match message.opcode {
				Opcode::Text => {
					sender.send_message(&Message::text(message.payload)).unwrap();
				}
				Opcode::Binary => {
					sender.send_message(&Message::binary(message.payload)).unwrap();
				}
				Opcode::Close => {
					let _ = sender.send_message(&Message::close());
					break;
				}
				Opcode::Ping => {
					sender.send_message(&Message::pong(message.payload)).unwrap();
				}
				_ => (),
			}
		}

		current_case_id += 1;
	}

	update_reports(addr.clone(), agent);
}

fn get_case_count(addr: String) -> usize {
	let url = addr + "/getCaseCount";
	let ws_uri = Url::parse(&url[..]).unwrap();
	let request = Client::connect(ws_uri).unwrap();
	let response = request.send().unwrap();
	match response.validate() {
		Ok(()) => (),
		Err(e) => {
			println!("{:?}", e);
			return 0;
		}
	}
	let (mut sender, mut receiver) = response.begin().split();

	let mut count = 0;

	for message in receiver.incoming_messages() {
		let message = match message {
			Ok(message) => message,
			Err(e) => {
				println!("Error: {:?}", e);
				let _ = sender.send_message(&Message::close_because(1002, "".to_string()));
				break;
			}
		};
		match message.opcode {
			Opcode::Text => {
				count = json::decode(&message.payload[..]).unwrap();
				println!("Will run {} cases...", count);
			}
			Opcode::Close => {
				let _ = sender.send_message(&Message::close());
				break;
			}
			Opcode::Ping => {
				sender.send_message(&Message::pong(message.payload)).unwrap();
			}
			_ => (),
		}
	}

	count
}

fn update_reports(addr: String, agent: &str) {
	let url = addr + "/updateReports?agent=" + agent;
	let ws_uri = Url::parse(&url[..]).unwrap();
	let request = Client::connect(ws_uri).unwrap();
	let response = request.send().unwrap();
	match response.validate() {
		Ok(()) => (),
		Err(e) => {
			println!("{:?}", e);
			return;
		}
	}
	let (mut sender, mut receiver) = response.begin().split();

	println!("Updating reports...");

	for message in receiver.incoming_messages() {
		let message: Message = match message {
			Ok(message) => message,
			Err(e) => {
				println!("Error: {:?}", e);
				let _ = sender.send_message(&Message::close());
				return;
			}
		};
		match message.opcode {
			Opcode::Close => {
				let _ = sender.send_message(&Message::close());
				println!("Reports updated.");
				println!("Test suite finished!");
				return;
			}
			Opcode::Ping => {
				sender.send_message(&Message::pong(message.payload)).unwrap();
			}
			_ => (),
		}
	}
}
