use super::kind::PacketType;
use super::PayloadTrait;

#[derive(Debug, Nom, Serialize)]
#[nom(BigEndian)]
pub struct Rename {
	pub id: u32,
	#[nom(Parse(crate::util::parse_string))]
	#[serde(serialize_with = "crate::util::str_with_u32_length")]
	pub oldpath: String,
	#[nom(Parse(crate::util::parse_string))]
	#[serde(serialize_with = "crate::util::str_with_u32_length")]
	pub newpath: String
}

impl PayloadTrait for Rename {
	const Type: PacketType = PacketType::Rename;
	fn binsize(&self) -> u32 {
		4 + (4 + self.oldpath.len() as u32) + (4 + self.newpath.len() as u32)
	}
}

