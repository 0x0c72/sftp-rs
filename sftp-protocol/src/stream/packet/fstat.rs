use super::kind::PacketType;
use super::PayloadTrait;

#[derive(Debug, Nom, Serialize)]
#[nom(BigEndian)]
pub struct Fstat {
	pub id: u32,
	#[nom(Parse(crate::util::parse_uuid))]
	#[serde(serialize_with = "crate::util::serialize_uuid")]
	pub handle: uuid::Uuid
}

impl PayloadTrait for Fstat {
	const Type: PacketType = PacketType::Fstat;
	fn binsize(&self) -> u32 {
		4 + (4 + 36)
	}
}

