use crate::common::FileAttributes;

use super::kind::PacketType;
use super::PayloadTrait;

#[derive(Debug, Nom, Serialize)]
#[nom(BigEndian)]
pub struct Attrs {
	pub id: u32,
	pub attrs: FileAttributes
}

impl PayloadTrait for Attrs {
	const Type: PacketType = PacketType::Attrs;
	fn binsize(&self) -> u32 {
		4 + self.attrs.binsize()
	}
}

