
use log::warn;

use tokio;

use adrenaline::{Adrenaline, Configuration};

#[tokio::main]
async fn main() {
	env_logger::init();
	let ad = Adrenaline::new(Configuration::
	new_with_remote_address("0.0.0.0:8080"));

	let response = ad.send_file("examples/image.png".to_string()).await;
	match response {
		Ok(_x) => {
			println!("Message sent successfully!");
		},
		Err(e) => {
			warn!("{}", e);
		}
	}

}