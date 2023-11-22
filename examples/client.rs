use std::time::{Duration, SystemTime};
use adrenaline::{Adrenaline, Configuration};
use tokio;

#[tokio::main]
async fn main() {
	let ad = Adrenaline::new(Configuration::
	new_with_remote_address("0.0.0.0:8080"));

	let mut start = SystemTime::now();
	loop {
		if start.elapsed().unwrap() > Duration::from_secs(5) {
			break
		}
		ad.send_string("Test").await;
	}
}