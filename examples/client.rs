use adrenaline::{Adrenaline, Configuration};
use tokio;

#[tokio::main]
async fn main() {
	let ad = Adrenaline::new(Configuration::
	new_with_remote_address("0.0.0.0:8080"));
	ad.send_string("Test").await;
}