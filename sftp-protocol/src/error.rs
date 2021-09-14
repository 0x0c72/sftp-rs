#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("error decoding packet header")]
	PacketHeader,
	#[error("no header")]
	NoHeader,
	#[error("error decoding packet")]
	Packet,
	#[error("invalid path")]
	InvalidPath,
	#[error("I/O failure")]
	IO(#[from] std::io::Error),
	#[error("metadata error")]
	Metadata(#[from] nix::Error)
}

