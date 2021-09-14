use serde::Serialize;

use crate::common::FileAttributes;

pub mod kind;
use kind::PacketType;

pub mod init;
use init::Init;
pub mod version;
use version::Version;
pub mod open;
use open::Open;
pub mod close;
use close::Close;
pub mod read;
use read::Read;
pub mod write;
use write::Write;
pub mod lstat;
use lstat::Lstat;
pub mod fstat;
use fstat::Fstat;
pub mod setstat;
use setstat::SetStat;
pub mod fsetstat;
use fsetstat::FSetStat;
pub mod opendir;
use opendir::OpenDir;
pub mod readdir;
use readdir::ReadDir;
pub mod remove;
use remove::Remove;
pub mod mkdir;
use mkdir::MkDir;
pub mod rmdir;
use rmdir::RmDir;
pub mod realpath;
use realpath::RealPath;
pub mod stat;
use stat::Stat;
pub mod rename;
use rename::Rename;
pub mod readlink;
use readlink::ReadLink;
pub mod symlink;
use symlink::Symlink;
pub mod status;
use status::Status;
use status::StatusType;
pub mod handle;
use handle::Handle;
pub mod data;
use data::Data;
pub mod name;
use name::Name;
pub mod attrs;
use attrs::Attrs;
pub mod extended;
use extended::Request as ExtendedRequest;
use extended::Response as ExtendedResponse;

#[derive(Debug, Nom, Serialize)]
#[nom(BigEndian)]
pub struct PacketRaw<'a> {
	pub length: u32,
	pub kind: PacketType,
	#[nom(Take="i.len()")]
	pub raw_payload: &'a [u8]
}

#[derive(Debug, Nom, Serialize)]
#[nom(BigEndian)]
pub struct PacketHeader {
	pub length: u32,
	pub kind: PacketType
}

#[derive(Debug, Nom, Serialize)]
#[nom(BigEndian)]
pub struct Packet {
	pub header: PacketHeader,
	#[nom(Parse="{ |i| Payload::parse(i, header.kind) }")]
	pub payload: Payload
}

#[enum_dispatch]
pub trait PayloadTrait : Serialize + Into<Payload> {
	const Type: PacketType;
	fn binsize(&self) -> u32;

	fn header(&self) -> PacketHeader {
		PacketHeader{
			length: 1 + self.binsize(),
			kind: Self::Type
		}
	}

	fn into_packet(self) -> Packet {
		Packet{
			header: self.header(),
			payload: self.into()
		}
	}
}

#[enum_dispatch(PayloadTrait(Type:PacketType=PacketType::UNIMPLEMENTED))]
#[derive(Debug, Nom, Serialize)]
#[nom(Selector = "PacketType")]
#[serde(untagged)]
pub enum Payload {
	#[nom(Selector = "PacketType::Init")]
	Init(Init),
	#[nom(Selector = "PacketType::Version")]
	Version(Version),
	#[nom(Selector = "PacketType::Open")]
	Open(Open),
	#[nom(Selector = "PacketType::Close")]
	Close(Close),
	#[nom(Selector = "PacketType::Read")]
	Read(Read),
	#[nom(Selector = "PacketType::Write")]
	Write(Write),
	#[nom(Selector = "PacketType::Lstat")]
	Lstat(Lstat),
	#[nom(Selector = "PacketType::Fstat")]
	Fstat(Fstat),
	#[nom(Selector = "PacketType::SetStat")]
	SetStat(SetStat),
	#[nom(Selector = "PacketType::FSetStat")]
	FSetStat(FSetStat),
	#[nom(Selector = "PacketType::OpenDir")]
	OpenDir(OpenDir),
	#[nom(Selector = "PacketType::ReadDir")]
	ReadDir(ReadDir),
	#[nom(Selector = "PacketType::Remove")]
	Remove(Remove),
	#[nom(Selector = "PacketType::MkDir")]
	MkDir(MkDir),
	#[nom(Selector = "PacketType::RmDir")]
	RmDir(RmDir),
	#[nom(Selector = "PacketType::RealPath")]
	RealPath(RealPath),
	#[nom(Selector = "PacketType::Stat")]
	Stat(Stat),
	#[nom(Selector = "PacketType::Rename")]
	Rename(Rename),
	#[nom(Selector = "PacketType::ReadLink")]
	ReadLink(ReadLink),
	#[nom(Selector = "PacketType::Symlink")]
	Symlink(Symlink),
	#[nom(Selector = "PacketType::Status")]
	Status(Status),
	#[nom(Selector = "PacketType::Handle")]
	Handle(Handle),
	#[nom(Selector = "PacketType::Data")]
	Data(Data),
	#[nom(Selector = "PacketType::Name")]
	Name(Name),
	#[nom(Selector = "PacketType::Attrs")]
	Attrs(Attrs),
	#[nom(Selector = "PacketType::Extended")]
	Extended(ExtendedRequest),
	#[nom(Selector = "PacketType::ExtendedReply")]
	ExtendedReply(ExtendedResponse)
}

impl Payload {
	pub fn init(version: u32, extension_data: Vec<u8>) -> Self {
		Self::Init(Init{
			version: version,
			extension_data: extension_data
		})
	}

	pub fn version(version: u32, extension_data: Vec<u8>) -> Self {
		Self::Version(Version{
			version: version,
			extension_data: extension_data
		})
	}

	pub fn real_path(id: u32, path: impl AsRef<str>) -> Self {
		Self::RealPath(RealPath{
			id: id,
			path: path.as_ref().to_string()
		})
	}

	pub fn status(id: u32, status: StatusType, message: impl AsRef<str>) -> Self {
		Self::Status(Status{
			id: id,
			status: status,
			message: message.as_ref().to_string(),
			language: "en-US".to_string()
		})
	}

	pub fn handle(id: u32) -> Handle {
		Handle{
			id: id,
			handle: uuid::Uuid::new_v4()
		}
	}

	pub fn name(id: u32) -> Name {
		Name{
			id: id,
			files: Vec::with_capacity(1)
		}
	}

	pub fn attrs(id: u32) -> Attrs {
		Attrs{
			id: id,
			attrs: FileAttributes::new()
		}
	}

	// TODO:  Configurable limit for size
	pub fn data_with_size(id: u32, size: u32) -> Data {
		Data{
			id: id,
			data: vec![0; size as usize]
		}
	}
}


