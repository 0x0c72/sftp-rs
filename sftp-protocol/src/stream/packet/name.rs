use crate::common::FileAttributes;

use super::kind::PacketType;
use super::PayloadTrait;

#[derive(Clone, Debug, Nom, Serialize)]
#[nom(BigEndian)]
pub struct Name {
	pub id: u32,
	#[serde(serialize_with = "crate::util::vec_with_u32_length")]
	pub files: Vec<File>
}

impl Name {
	pub fn new(id: u32) -> Self {
		Self{
			id: id,
			files: Vec::new()
		}
	}

	pub fn append_file(&mut self, filename: &str, longname: &str, attrs: FileAttributes) {
		self.files.push(File{
			filename: filename.to_string(),
			longname: longname.to_string(),
			attrs: attrs
		});
	}
}

impl PayloadTrait for Name {
	const Type: PacketType = PacketType::Name;
	fn binsize(&self) -> u32 {
		4 + 4 + self.files.iter().map(|f| f.binsize()).sum::<u32>()
	}
}

#[derive(Clone, Debug, Nom, Serialize)]
#[nom(BigEndian)]
pub struct File {
	#[nom(Parse(crate::util::parse_string))]
	#[serde(serialize_with = "crate::util::str_with_u32_length")]
	pub filename: String,
	#[nom(Parse(crate::util::parse_string))]
	#[serde(serialize_with = "crate::util::str_with_u32_length")]
	pub longname: String,
	pub attrs: FileAttributes
}

impl File {
	pub fn binsize(&self) -> u32 {
		4 + self.filename.len() as u32 + 4 + self.longname.len() as u32 + self.attrs.binsize()
	}
}

