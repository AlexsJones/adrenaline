use std::error::Error;
use std::io::{Read, Write};
use std::io;

use log::info;
use crate::Packet;

const UDP_HEADER: usize = 8;
// 0,0,0,0,0,0,0,0 DEFAULT
// 0,0,0,0,0,0,0,1 START
// 0,0,0,0,1,1,1,1 CONTINUE
// 0,0,0,1,1,1,1,1 SINGLE_UNIT
// 0,0,0,0,0,0,1,1 END
// 0,0,0,0,0,1,1,1 ERROR
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum ControlCommand {
	START,
	CONTINUE,
	SINGLE_UNIT,
	END,
	ERROR
}
pub(crate) const MAX_USER_CONTROL_HEADER: usize = 8;
const IP_HEADER: usize = 20;
const AG_HEADER: usize = 4;
const MAX_DATA_LENGTH: usize = (9216 - 1) - UDP_HEADER - IP_HEADER - MAX_USER_CONTROL_HEADER;
pub const MAX_CHUNK_SIZE: usize = MAX_DATA_LENGTH - AG_HEADER;

pub(crate) const MAX_DATAGRAM_SIZE: usize = 9216;
pub struct FileChunkInfo {
	pub size: usize,
	pub chunks: Vec<Vec<u8>>
}

pub fn get_chunks_from_file(filename: String) -> Result<FileChunkInfo, io::Error> {
	let mut f = std::fs::File::open(filename)?;
	let mut list_of_chunks = Vec::new();
	let mut chunkInfo = FileChunkInfo{ size: 0, chunks: vec![] };
	loop {
		let mut chunk = Vec::with_capacity(MAX_CHUNK_SIZE);
		let n = std::io::Read::by_ref(&mut f).take(MAX_CHUNK_SIZE as u64).read_to_end(&mut chunk)?;
		chunkInfo.size += n;
		if n == 0 {
			break;
		}
		//let start:usize = if list_of_chunks.len() != 0 { 0 } else { 0x20 }; // skip header
		for i in 0..n {
			chunk[i] = !chunk[i]; // neg
		}
		list_of_chunks.push(chunk);
		if n < MAX_CHUNK_SIZE {
			break;
		}
	}
	chunkInfo.chunks = list_of_chunks.clone();
	Ok(chunkInfo)
}

pub fn create_file_from_packets(packet: &Vec<Packet>) -> Result<(),Box<dyn Error>> {

	let mut file_buffer_vec: Vec<u8> = vec![];

	for p in packet {
		file_buffer_vec.extend(p.bytes.clone());
	}

	let mut f = std::fs::File::create("output")?;
	f.write_all(&file_buffer_vec)?;

	Ok(())
}

pub fn create_control_header(c: ControlCommand) -> [u8; MAX_USER_CONTROL_HEADER] {
	match c {
		ControlCommand::START => {
			[0,0,0,0,0,0,0,1]
		},
		ControlCommand::CONTINUE => {
			[0,0,0,0,1,1,1,1]
		},
		ControlCommand::SINGLE_UNIT => {
			[0,0,0,1,1,1,1,1]
		}
		ControlCommand::END => {
			[0,0,0,0,0,0,1,1]
		}
		ControlCommand::ERROR => {
			[0,0,0,0,0,1,1,1]
		}
	}
}

pub fn get_command_from_control_header(input: &[u8]) -> ControlCommand {

	if input == [0,0,0,0,0,0,0,1] {
		info!("Found flow control START");
		return ControlCommand::START
	}
	if input == [0,0,0,0,1,1,1,1] {
		info!("Found flow control CONTINUE");
		return ControlCommand::CONTINUE
	}
	if input == [0,0,0,1,1,1,1,1] {
		info!("Found flow control SINGLE_UNIT");
		return ControlCommand::SINGLE_UNIT
	}
	if input == [0,0,0,0,0,0,1,1] {
		info!("Found flow control END");
		return ControlCommand::END
	}
	info!("Found flow control Error");
	ControlCommand::ERROR
}