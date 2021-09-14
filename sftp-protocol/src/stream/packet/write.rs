use super::kind::PacketType;
use super::PayloadTrait;

#[derive(Nom, Serialize)]
#[nom(BigEndian)]
pub struct Write {
	pub id: u32,
	#[nom(Parse(crate::util::parse_uuid))]
	#[serde(serialize_with = "crate::util::serialize_uuid")]
	pub handle: uuid::Uuid,
	pub offset: u64,
	#[nom(Parse(crate::util::parse_u8_vec))]
	#[serde(serialize_with = "crate::util::vec_with_u32_length")]
	pub data: Vec<u8>,
}

impl PayloadTrait for Write {
	const Type: PacketType = PacketType::Write;
	fn binsize(&self) -> u32 {
		4 + (4 + 36) + 8 + (4 + self.data.len() as u32)
	}
}

impl std::fmt::Debug for Write {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Write")
			.field("id", &self.id)
			.field("handle", &self.handle)
			.field("offset", &self.offset)
			.field("data", &format!("[u8; {}]", self.data.len()))
			.finish()
	}
}

