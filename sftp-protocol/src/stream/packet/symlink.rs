use super::kind::PacketType;
use super::PayloadTrait;

#[derive(Debug, Nom, Serialize)]
#[nom(BigEndian)]
pub struct Symlink {
	pub id: u32,
	#[nom(Parse(crate::util::parse_string))]
	#[serde(serialize_with = "crate::util::str_with_u32_length")]
	pub linkpath: String,
	#[nom(Parse(crate::util::parse_string))]
	#[serde(serialize_with = "crate::util::str_with_u32_length")]
	pub targetpath: String
}

impl PayloadTrait for Symlink {
	const Type: PacketType = PacketType::Symlink;
	fn binsize(&self) -> u32 {
		4 + (4 + self.linkpath.len() as u32) + (4 + self.targetpath.len() as u32)
	}
}

