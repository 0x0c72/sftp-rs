use super::kind::PacketType;
use super::PayloadTrait;

#[derive(Debug, Nom, Serialize)]
#[nom(BigEndian)]
pub struct Status {
	pub id: u32,
	pub status: StatusType,
	// TODO:  Do we need to do anything special for ISO-10646 UTF-8 encoding here?
	#[nom(Parse(crate::util::parse_string))]
	#[serde(serialize_with = "crate::util::str_with_u32_length")]
	pub message: String,
	#[nom(Parse(crate::util::parse_string))]
	#[serde(serialize_with = "crate::util::str_with_u32_length")]
	pub language: String
}

impl PayloadTrait for Status {
	const Type: PacketType = PacketType::Status;
	fn binsize(&self) -> u32 {
		4 + 4 + (4 + self.message.len() as u32) + (4 + self.language.len() as u32)
	}
}

#[derive(Clone, Copy, Debug, Nom, Serialize_repr)]
#[repr(u32)]
pub enum StatusType {
	OK = 0,
	EOF = 1,
	NoSuchFile = 2,
	PermissionDenied = 3,
	Failure = 4,
	BadMessage = 5,
	NoConnection = 6,
	ConnectionLost = 7,
	OpUnsupported = 8
}

