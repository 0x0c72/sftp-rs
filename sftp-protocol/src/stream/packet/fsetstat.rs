use crate::common::FileAttributes;

use super::kind::PacketType;
use super::PayloadTrait;

#[derive(Debug, Nom, Serialize)]
#[nom(BigEndian)]
pub struct FSetStat {
	pub id: u32,
	#[nom(Parse(crate::util::parse_uuid))]
	#[serde(serialize_with = "crate::util::serialize_uuid")]
	pub handle: uuid::Uuid,
	pub attrs: FileAttributes
}

impl PayloadTrait for FSetStat {
	const Type: PacketType = PacketType::FSetStat;
	fn binsize(&self) -> u32 {
		4 + (4 + 36) + self.attrs.binsize()
	}
}

