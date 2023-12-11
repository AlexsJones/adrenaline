mod support;

use std::error::Error;
use std::net::SocketAddr;

use log::{debug, info};
use tokio::net::UdpSocket;
use crate::support::{ControlCommand, create_control_header, create_file_from_packets, get_chunks_from_file, get_command_from_control_header};

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
	cpu_count: usize,
	buffer: Vec<Packet>
}

pub struct Packet {
	pub control_header: ControlCommand,
	pub bytes: Vec<u8>,
	pub len: usize,
	pub remote_address: SocketAddr,
}

impl Adrenaline {
	pub fn new(config: Configuration) -> Self {
		let num_cpus = num_cpus::get();
		info!("{} cores available", num_cpus);
		info!("Max datagram size {}", support::MAX_DATAGRAM_SIZE);
		info!("Max chunk size {}", support::MAX_CHUNK_SIZE);
		Self {
			configuration: config,
			cpu_count: num_cpus,
			buffer: vec![],
		}
	}
	pub fn new_udp_reuse_port(&self, local_addr: SocketAddr) -> UdpSocket {
		let udp_sock = socket2::Socket::new(
			if local_addr.is_ipv4() {
				socket2::Domain::IPV4
			} else {
				socket2::Domain::IPV6
			},
			socket2::Type::DGRAM,
			None,
		)
			.unwrap();
		udp_sock.set_reuse_port(true).unwrap();
		udp_sock.set_cloexec(true).unwrap();
		udp_sock.set_nonblocking(true).unwrap();
		udp_sock.bind(&socket2::SockAddr::from(local_addr)).unwrap();
		let udp_sock: std::net::UdpSocket = udp_sock.into();
		udp_sock.try_into().unwrap()
	}

	pub async fn serve(
		&mut self,
		_callback: fn(packet: Packet) -> Option<Vec<u8>>,
	) {

			let socket = self.new_udp_reuse_port(self.configuration.local_address);

				let mut buf = [0; support::MAX_CHUNK_SIZE + support::MAX_USER_CONTROL_HEADER];
				loop {
					let (len, _addr) = socket.recv_from(&mut buf).await.unwrap();
					debug!("Received packet of length {} bytes", len);

					let s = buf.split_at(8);
					let control_header = get_command_from_control_header(s.0);
					// reconstruct into a packet
					let mut data = Vec::new();
					data.extend_from_slice(s.1);

					let inbound_packet = Packet {
						control_header,
						bytes: data ,
						len: len - support::MAX_USER_CONTROL_HEADER,
						remote_address: self.configuration.local_address,
					};
					debug!("Inbound packet chunk size {}", inbound_packet.bytes.len());

					match get_command_from_control_header(s.0) {
						ControlCommand::START => {
							debug!("Found flow control Start");
							self.buffer.clear();
							self.buffer.push(inbound_packet);
						}
						ControlCommand::CONTINUE => {
							debug!("Found flow control Continue");
							self.buffer.push(inbound_packet);
						}
						ControlCommand::SINGLE_UNIT => {
							debug!("Found flow control Single Unit");
							self.buffer.clear();
							self.buffer.push(inbound_packet);
							create_file_from_packets(&self.buffer);
						}
						ControlCommand::END => {
							debug!("Found flow control End");
							self.buffer.push(inbound_packet);
							create_file_from_packets(&self.buffer);
						}
						ControlCommand::ERROR => {}
					}

				}
	}

	async fn send_packet(&self, packet: Packet) -> Result<(), Box<dyn Error>> {
		let socket = UdpSocket::bind(self.configuration.local_address).await?;
		socket.connect(&self.configuration.remote_address).await?;

		// We must packet the control header into the the bytes body
		let control_header_bytes = create_control_header(packet.control_header).to_vec();
		let body = [ control_header_bytes,packet.bytes].concat();
		debug!("Sending of combined body of size {} bytes", body.len());
		socket.send(body.as_slice()).await?;
		Ok(())
	}
	pub async fn send_file(&self, file_name: String) ->  Result<(),Box<dyn Error>> {
		// Chunk file
		info!("Sending file {}", file_name);
		let chunks = get_chunks_from_file(file_name);
		match chunks {
			Ok(mut x) => {
				info!("File size is {}, chunking into {} chunks of size {}",x.size, x.chunks.len(), support::MAX_CHUNK_SIZE);
				// Create control header
				// Cycle through all the chunks, create packets and assign their control headers...
				if x.chunks.len() == 1 {
					// Send a single packet with single chunk
					let send_packet = Packet{
						control_header: ControlCommand::SINGLE_UNIT,
						bytes: x.chunks[0].clone(),
						len: x.size,
						remote_address: self.configuration.remote_address,
					};
					debug!("Raw Chunk size {}", &send_packet.bytes.len());
					self.send_packet(send_packet).await?;
					return Ok(())
				}

				// Get start and end chunks
				let chunk_len = x.chunks.len();
				let start_chunk:Vec<Vec<u8>> = {
					x.chunks.drain(0..1).collect::<Vec<_>>() // Mutable borrow is scoped and ends here
				};
				let start_chunk_len = start_chunk.first().unwrap().len();

				let end_chunk = x.chunks.remove(chunk_len - 2);
				let end_chunk_len = end_chunk.to_vec().len();
				// Create start packet
				let start_packet = Packet{
					control_header: ControlCommand::START,
					bytes: start_chunk.into_iter().flatten().collect(),
					len: start_chunk_len,
					remote_address: self.configuration.remote_address,
				};

				debug!("Start Raw Chunk size {}", &start_packet.bytes.len());
				self.send_packet(start_packet).await?;

				for middle_chunk in x.chunks {
					let len = middle_chunk.len();
					let middle_packet = Packet{
						control_header: ControlCommand::CONTINUE,
						bytes: middle_chunk,
						len,
						remote_address: self.configuration.remote_address,
					};
					debug!("Middle Raw Chunk size {}", &middle_packet.bytes.len());
					self.send_packet(middle_packet).await?;
				}

				let end_packet = Packet{
					control_header: ControlCommand::END,
					bytes: end_chunk,
					len: end_chunk_len,
					remote_address: self.configuration.remote_address,

				};
				debug!("End Chunk size {}", &end_packet.bytes.len());
				self.send_packet(end_packet).await?;

			},
			Err(e) => {
				info!("{}",e);
			}
		}

		Ok(())
	}
}


#[cfg(test)]
mod tests {
	
	use crate::{Adrenaline, Configuration};
	#[tokio::test]
	#[should_panic]
	async fn test_local_address_not_parsed() {
		let conf = Configuration::new_with_local_address("");
		let _adrenaline = Adrenaline::new(conf);
	}
	#[tokio::test]
	#[should_panic]
	async fn test_remote_address_not_parsed() {
		let conf = Configuration::new_with_remote_address("");
		let _adrenaline = Adrenaline::new(conf);
	}
}