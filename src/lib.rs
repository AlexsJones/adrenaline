use std::error::Error;
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use std::str;
pub struct Configuration {
	local_address: SocketAddr,
	remote_address: SocketAddr
}

impl Configuration {
	pub fn new_with_local_address(local_address: &str) -> Self {
		Self {
			local_address: local_address.parse().unwrap(),
			remote_address: "0.0.0.0:0".parse().unwrap()
		}
	}
	pub fn new_with_remote_address(remote_address: &str) -> Self {
		Self {
			local_address: "0.0.0.0:0".parse().unwrap(),
			remote_address: remote_address.parse().unwrap()
		}
	}
	pub fn new_with_addresses(local_address: &str, remote_address: &str) -> Self {
		Self {
			local_address: local_address.parse().unwrap(),
			remote_address: remote_address.parse().unwrap()
		}
	}
}
pub struct Adrenaline {
	configuration: Configuration
}

impl Adrenaline {
	pub fn new(config: Configuration) -> Self {
		Self {
			configuration: config
		}
	}
	pub async fn serve(&self, callback: fn(message: &str))  {
		let socket = UdpSocket::bind(&self.configuration.local_address).await.unwrap();
		let mut buf = [0; 1024];
		loop {
			let (len, addr) = socket.recv_from(&mut buf).await.unwrap();
			let message = str::from_utf8(&buf[..len]);
			match message {
				Ok(x) => {
					callback(x);
				},
				Err(e) => {}
			}
		}
	}
	pub async fn send_string(&self,payload: &str) -> Result<Option<Vec<u8>>,Box<dyn Error>> {
		let socket = UdpSocket::bind(self.configuration.local_address).await?;
		const MAX_DATAGRAM_SIZE: usize = 65_507;
		socket.connect(&self.configuration.remote_address).await?;
		socket.send(&payload.as_bytes()).await?;
		Ok(None)
	}
}