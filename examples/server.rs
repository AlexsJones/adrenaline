use std::error::Error;
use adrenaline::{Adrenaline, Configuration};

#[tokio::main]
async fn main() {
	let ad = Adrenaline::new(Configuration::new_with_local_address("0.0.0.0:8080"));
	ad.serve(|x| println!("{:?}", x)).await;
}