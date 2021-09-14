use super::kind::PacketType;
use super::PayloadTrait;

#[derive(Debug, Nom, Serialize)]
#[nom(BigEndian)]
pub struct Read {
	pub id: u32,
	#[nom(Parse(crate::util::parse_uuid))]
	#[serde(serialize_with = "crate::util::serialize_uuid")]
	pub handle: uuid::Uuid,
	pub offset: u64,
	pub len: u32,
}

impl PayloadTrait for Read {
	const Type: PacketType = PacketType::Read;
	fn binsize(&self) -> u32 {
		4 + (4 + 36) + 8 + 4
	}
}

