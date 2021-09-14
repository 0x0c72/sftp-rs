use std::io;
use std::io::Write;

use circular::Buffer;

pub mod packet;
use packet::Packet;
use packet::PacketHeader;

use crate::Error;

pub struct Stream {
	buffer: Buffer,
	current_header: Option<PacketHeader>
}

impl Write for Stream {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		self.buffer.write(buf)
	}

	fn flush(&mut self) -> io::Result<()> {
		Ok(())
	}
}

impl Stream {
	fn new() -> Self {
		Self{
			buffer: Buffer::with_capacity(1 * 1024 * 1024),
			current_header: None
		}
	}

	fn check_packet(&mut self) -> bool {
		match &self.current_header {
			None => {
				if(self.buffer.available_data() >= 5) {
					let header_bytes = &self.buffer.data()[..5];
					self.current_header = Some(PacketHeader::parse(header_bytes).unwrap().1);
					self.buffer.consume_noshift(5);
				}
				self.check_packet()
			},
			Some(header) => {
				self.buffer.available_data() >= header.length as usize
			}
		}
	}

	fn get_packet(&mut self) -> Result<Packet, Error> {
		match &self.current_header {
			None => Err(Error::NoHeader),
			Some(header) => {
				let len = header.length as usize - 1; // Exclude type byte, we already have it
				let data = &self.buffer.data()[..len];
				let result = match Packet::parse(data) {
					Ok(v) => Ok(v.1),
					Err(_) => Err(Error::Packet)
				};
				self.buffer.consume(len);
				result
			}
		}
	}
}

