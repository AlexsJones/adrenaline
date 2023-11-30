use std::string;

use adrenaline::{Adrenaline, Configuration};

#[tokio::main]
async fn main() {
	env_logger::init();
	let mut ad = Adrenaline::new(Configuration::new_with_local_address("0.0.0.0:8080"));
	ad.serve(|packet| {
		let resp_string = string::String::from_utf8_lossy(&packet.bytes);
		println!("Incoming packet from {}:{}",
		         packet.remote_address.ip().to_string(), packet.remote_address.port());
		if resp_string.contains("Test") {
			return Some("OK".as_bytes().to_vec());
		}
		None
	}).await;
}