use std::error::Error;
use adrenaline::{Adrenaline, Configuration};
use std::str;
use std::thread::sleep;
use std::time::Duration;

#[tokio::main]
async fn main() {
	let ad = Adrenaline::new(Configuration::new_with_local_address("0.0.0.0:8080"));

	ad.serve(|x,y|
		{
			let message = str::from_utf8(&x[..y]);
			println!("{}", message.unwrap())

		}).await;

}