use super::kind::PacketType;
use super::PayloadTrait;

#[derive(Debug, Nom, Serialize)]
#[nom(BigEndian)]
pub struct ReadLink {
	pub id: u32,
	#[nom(Parse(crate::util::parse_string))]
	#[serde(serialize_with = "crate::util::str_with_u32_length")]
	pub path: String
}

impl PayloadTrait for ReadLink {
	const Type: PacketType = PacketType::ReadLink;
	fn binsize(&self) -> u32 {
		4 + 4 + self.path.len() as u32
	}
}

