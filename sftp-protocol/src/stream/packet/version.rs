use super::kind::PacketType;
use super::PayloadTrait;

#[derive(Debug, Nom, Serialize)]
#[nom(BigEndian)]
pub struct Version {
	pub version: u32,
	#[serde(serialize_with = "crate::util::vec_u8_as_slice")]
	pub extension_data: Vec<u8>
}

impl PayloadTrait for Version {
	const Type: PacketType = PacketType::Version;
	fn binsize(&self) -> u32 {
		4 + self.extension_data.len() as u32
	}
}

