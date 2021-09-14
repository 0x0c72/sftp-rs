use nom::IResult;
use nom::number::streaming::be_u32;

use crate::common::FileAttributes;

use super::kind::PacketType;
use super::PayloadTrait;

#[derive(Debug, Nom, Serialize)]
#[nom(BigEndian)]
pub struct Open {
	pub id: u32,
	#[nom(Parse(crate::util::parse_string))]
	#[serde(serialize_with = "crate::util::str_with_u32_length")]
	pub path: String,
	pub pflags: OpenFlags,
	pub attrs: FileAttributes
}

impl PayloadTrait for Open {
	const Type: PacketType = PacketType::Open;
	fn binsize(&self) -> u32 {
		4 + (4 + self.path.len() as u32) + 4 + self.attrs.binsize()
	}
}

bitflags! {
	#[derive(Default, Serialize)]
	pub struct OpenFlags: u32 {
		const Read = 0x01;
		const Write = 0x02;
		const Append = 0x04;
		const Create = 0x08;
		const Truncate = 0x10;
		const Exclude = 0x20;
	}
}

impl OpenFlags {
	pub fn parse(i: &[u8]) -> IResult<&[u8], Self> {
		let (i, flags) = be_u32(i)?;
		let this = Self::from_bits_truncate(flags);
		Ok((i, this))
	}
}

