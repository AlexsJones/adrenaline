use std::error::Error;
use std::net::SocketAddr;
use std::sync::{Arc, mpsc};
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, SystemTime};

use timer;
use tokio::net::UdpSocket;
use tokio::task;

pub struct Configuration {
	local_address: SocketAddr,
	remote_address: SocketAddr,
}

impl Configuration {
	pub fn new_with_local_address(local_address: &str) -> Self {
		Self {
			local_address: local_address.parse().unwrap(),
			remote_address: "0.0.0.0:0".parse().unwrap(),
		}
	}
	pub fn new_with_remote_address(remote_address: &str) -> Self {
		Self {
			local_address: "0.0.0.0:0".parse().unwrap(),
			remote_address: remote_address.parse().unwrap(),
		}
	}
	pub fn new_with_addresses(local_address: &str, remote_address: &str) -> Self {
		Self {
			local_address: local_address.parse().unwrap(),
			remote_address: remote_address.parse().unwrap(),
		}
	}
}

pub struct Adrenaline {
	configuration: Configuration,
	tps: Arc<i32>,
}

pub struct Packet {
	pub bytes: [u8; 1024],
	pub len: usize,
}

impl Adrenaline {
	pub fn new(config: Configuration) -> Self {
		Self {
			configuration: config,
			tps: Arc::new(0),
		}
	}
	pub fn get_tps(&self) -> Arc<i32> {
		self.tps.clone()
	}
	pub async fn serve_with_channel(&self) -> Result<Receiver<Packet>, Box<dyn Error>> {
		let (tx, rx): (Sender<Packet>, Receiver<Packet>) = mpsc::channel();
		let socket = UdpSocket::bind(&self.configuration.local_address).await?;
		let mut buf = [0; 1024];
		tokio::spawn(async move {
			loop {
				let (len, addr) = socket.recv_from(&mut buf).await.unwrap();
				let bufc = buf.clone();
				let lenc = len.clone();
				let txc = tx.clone();
				task::spawn(async move{
					txc.send(Packet {
						bytes: bufc,
						len: lenc,
					});
				});
			}
		});
		Ok(rx)
	}
	pub async fn serve(&self, callback: fn(message: [u8; 1024], len: usize)) {
		let (tx, rx): (Sender<i32>, Receiver<i32>) = mpsc::channel();
		// run the timer on another thread
		tokio::spawn(async move {
			let mut start = SystemTime::now();
			let mut rps_counter = 0;
			loop {
				let resp = rx.try_recv();
				match resp {
					Ok(x) => {
						if x != 0 {
							rps_counter += x;
						}
					}
					Err(e) => {}
				}

				// get the number of transactions since the last tick
				if rps_counter != 0 {
					if start.elapsed().unwrap() >= Duration::from_secs(1) {
						start = SystemTime::now();
						rps_counter = 0;
					}
				}
			}
		});

		let socket = UdpSocket::bind(&self.configuration.local_address).await.unwrap();
		let mut buf = [0; 1024];
		loop {
			let (len, addr) = socket.recv_from(&mut buf).await.unwrap();
			tokio::spawn(async move {
				callback(buf.clone(), len);
			});
			tx.send(1).unwrap();
		}
	}
	pub async fn send_string(&self, payload: &str) -> Result<Option<Vec<u8>>, Box<dyn Error>> {
		let socket = UdpSocket::bind(self.configuration.local_address).await?;
		const MAX_DATAGRAM_SIZE: usize = 65_507;
		socket.connect(&self.configuration.remote_address).await?;
		socket.send(&payload.as_bytes()).await?;
		Ok(None)
	}
}