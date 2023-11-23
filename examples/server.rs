use adrenaline::{Adrenaline, Configuration};

#[tokio::main]
async fn main() {
	let ad = Adrenaline::new(Configuration::new_with_local_address("0.0.0.0:8080"));
	let rx = ad.serve_with_channel().await;
	match rx {
		Ok(x) => {
			loop {
				let received_packet = x.recv();
				match received_packet {
					Ok(y) => {
						println!("{:?}", y.len);
					}
					Err(e) => {
						println!("error {}", e);
					}
				}
			}
		},
		Err(e) => {
			panic!("{}",e);
		}
	}
}