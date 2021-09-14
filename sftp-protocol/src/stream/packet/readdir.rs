use super::kind::PacketType;
use super::PayloadTrait;

#[derive(Debug, Nom, Serialize)]
#[nom(BigEndian)]
pub struct ReadDir {
	pub id: u32,
	#[nom(Parse(crate::util::parse_uuid))]
	#[serde(serialize_with = "crate::util::serialize_uuid")]
	pub handle: uuid::Uuid
}

impl PayloadTrait for ReadDir {
	const Type: PacketType = PacketType::ReadDir;
	fn binsize(&self) -> u32 {
		4 + 4 + 36
	}
}

