use super::kind::PacketType;
use super::PayloadTrait;

#[derive(Debug, Nom, Serialize)]
#[nom(BigEndian)]
pub struct Request {
	pub id: u32,
	#[nom(Parse(crate::util::parse_string))]
	#[serde(serialize_with = "crate::util::str_with_u32_length")]
	pub request: String,
	#[serde(serialize_with = "crate::util::vec_with_u32_length")]
	pub data: Vec<u8>
}

impl PayloadTrait for Request {
	const Type: PacketType = PacketType::Extended;
	fn binsize(&self) -> u32 {
		4 + (4 + self.request.len() as u32) + (4 + self.data.len() as u32)
	}
}

#[derive(Debug, Nom, Serialize)]
#[nom(BigEndian)]
pub struct Response {
	pub id: u32,
	#[serde(serialize_with = "crate::util::vec_with_u32_length")]
	pub data: Vec<u8>
}

impl PayloadTrait for Response {
	const Type: PacketType = PacketType::ExtendedReply;
	fn binsize(&self) -> u32 {
		4 + (4 + self.data.len() as u32)
	}
}

