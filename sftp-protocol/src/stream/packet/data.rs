use super::kind::PacketType;
use super::PayloadTrait;

#[derive(Debug, Nom, Serialize)]
#[nom(BigEndian)]
pub struct Data {
	pub id: u32,
	#[serde(serialize_with = "crate::util::vec_with_u32_length")]
	pub data: Vec<u8>
}

impl PayloadTrait for Data {
	const Type: PacketType = PacketType::Data;
	fn binsize(&self) -> u32 {
		4 + (4 + self.data.len() as u32)
	}
}

