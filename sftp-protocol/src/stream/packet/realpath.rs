use super::kind::PacketType;
use super::PayloadTrait;

#[derive(Debug, Nom, Serialize)]
#[nom(BigEndian)]
pub struct RealPath {
	pub id: u32,
	#[nom(Parse(crate::util::parse_string))]
	#[serde(serialize_with = "crate::util::str_with_u32_length")]
	pub path: String
}

impl PayloadTrait for RealPath {
	const Type: PacketType = PacketType::RealPath;
	fn binsize(&self) -> u32 {
		4 + 4 + self.path.len() as u32
	}
}

