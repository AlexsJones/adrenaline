

use adrenaline::{Adrenaline, Configuration};

#[tokio::main]
async fn main() {
	env_logger::init();
	let mut ad = Adrenaline::new(Configuration::new_with_local_address("0.0.0.0:8080"));
	ad.serve(|packet| {
		println!("Incoming packet from {}:{}",
		         packet.remote_address.ip().to_string(), packet.remote_address.port());
		None
	}).await;
}