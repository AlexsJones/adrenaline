use std::error::Error;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::io;
use std::slice::Chunks;
use chrono::Local;

use log::{debug, info};
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
pub(crate) const MAX_DATAGRAM_SIZE: usize = 9216;
const MAX_DATA_LENGTH: usize = (MAX_DATAGRAM_SIZE - 1) - UDP_HEADER - IP_HEADER - MAX_USER_CONTROL_HEADER;
pub const MAX_CHUNK_SIZE: usize = MAX_DATA_LENGTH - AG_HEADER;

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

		list_of_chunks.push(chunk);
		if n < MAX_CHUNK_SIZE {
			break;
		}
	}
	chunkInfo.chunks = list_of_chunks.clone();
	Ok(chunkInfo)
}

pub fn chunk_to_file(filename: &str, chunk: Vec<u8>)  -> Result<(), io::Error>{
	let mut file = OpenOptions::new()
		.append(true) // Set the append flag
		.create(true) // Create the file if it does not exist
		.open(filename)?;

	writeln!(file, "{:?}\n\n\n", chunk)?;
	Ok(())
}

fn find_eight_consecutive_nulls(data: &Vec<u8>) -> Option<usize> {
	let mut count = 0;
	let mut start_index = None;

	for (index, &byte) in data.iter().enumerate() {
		if byte == 0 {
			if count == 0 {
				start_index = Some(index);
			}
			count += 1;

			if count >= 8 {
				return start_index;
			}
		} else {
			count = 0;
			start_index = None;
		}
	}

	None
}
fn filename_from_timestamp() -> String {
	let now = Local::now();
	format!("received_file{}", now.format("%Y%m%d_%H%M%S"))
}
pub fn create_file_from_packets(packet: &Vec<Packet>) -> Result<(),Box<dyn Error>> {

	let filename = filename_from_timestamp();
	let mut file_buffer_vec: Vec<u8> = vec![];

	for mut p in packet {
		file_buffer_vec.extend(&p.bytes);
	}
	let mut f = std::fs::File::create(&filename)?;
	f.write_all(&file_buffer_vec)?;

	info!("Wrote new file {}", &filename);
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
		return ControlCommand::START
	}
	if input == [0,0,0,0,1,1,1,1] {
		return ControlCommand::CONTINUE
	}
	if input == [0,0,0,1,1,1,1,1] {
		return ControlCommand::SINGLE_UNIT
	}
	if input == [0,0,0,0,0,0,1,1] {
		return ControlCommand::END
	}
	ControlCommand::ERROR
}