extern crate websocket;
extern crate rustc_serialize as serialize;

use std::str::from_utf8;
use websocket::ClientBuilder;
use websocket::Message;
use websocket::message::Type;
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
        let case_id = current_case_id;
        current_case_id += 1;
		    let url = addr.clone() + "/runCase?case=" + &case_id.to_string()[..] + "&agent=" + agent;

        let client = ClientBuilder::new(&url).unwrap()
            .connect_insecure().unwrap();

		    let (mut receiver, mut sender) = client.split().unwrap();

		    println!("Executing test case: {}/{}", case_id, case_count);

		    for message in receiver.incoming_messages() {
			      let message: Message = match message {
				        Ok(message) => message,
				        Err(e) => {
					          println!("Error: {:?}", e);
					          let _ = sender.send_message(&Message::close());
					          break;
				        }
			      };

			      match message.opcode {
				        Type::Text => {
                    let response = Message::text(from_utf8(&*message.payload).unwrap());
					          sender.send_message(&response).unwrap();
				        }
				        Type::Binary => {
					          sender.send_message(&Message::binary(message.payload)).unwrap();
				        }
				        Type::Close => {
					          let _ = sender.send_message(&Message::close());
					          break;
				        }
				        Type::Ping => {
					          sender.send_message(&Message::pong(message.payload)).unwrap();
				        }
				        _ => (),
			      }
		    }
	  }

	  update_reports(addr.clone(), agent);
}

fn get_case_count(addr: String) -> usize {
	  let url = addr + "/getCaseCount";

    let client = match ClientBuilder::new(&url).unwrap().connect_insecure() {
        Ok(c) => c,
		    Err(e) => {
			      println!("{:?}", e);
			      return 0;
		    },
    };

	  let (mut receiver, mut sender) = client.split().unwrap();

	  let mut count = 0;

	  for message in receiver.incoming_messages() {
		    let message: Message = match message {
			      Ok(message) => message,
			      Err(e) => {
				        println!("Error: {:?}", e);
				        let _ = sender.send_message(&Message::close_because(1002, "".to_string()));
				        break;
			      }
		    };
		    match message.opcode {
			      Type::Text => {
				        count = json::decode(from_utf8(&*message.payload).unwrap()).unwrap();
				        println!("Will run {} cases...", count);
			      }
			      Type::Close => {
				        let _ = sender.send_message(&Message::close());
				        break;
			      }
			      Type::Ping => {
				        sender.send_message(&Message::pong(message.payload)).unwrap();
			      }
			      _ => (),
		    }
	  }

	  count
}

fn update_reports(addr: String, agent: &str) {
	  let url = addr + "/updateReports?agent=" + agent;

    let client = match ClientBuilder::new(&url).unwrap().connect_insecure() {
        Ok(c) => c,
		    Err(e) => {
			      println!("{:?}", e);
			      return;
		    },
    };

	  let (mut receiver, mut sender) = client.split().unwrap();

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
			      Type::Close => {
				        let _ = sender.send_message(&Message::close());
				        println!("Reports updated.");
				        println!("Test suite finished!");
				        return;
			      }
			      Type::Ping => {
				        sender.send_message(&Message::pong(message.payload)).unwrap();
			      }
			      _ => (),
		    }
	  }
}
