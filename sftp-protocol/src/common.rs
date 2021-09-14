use nom::IResult;
use nom::number::streaming::be_u32;
use nom::number::streaming::be_u64;

use serde::ser::Serialize;
use serde::ser::Serializer;
use serde::ser::SerializeStruct;

mod metadata;
pub use metadata::Metadata;

bitflags! {
	#[derive(Default)]
	pub struct FileAttrFlags: u32 {
		const Size = 0x00000001;
		const UidGid = 0x00000002;
		const Permissions = 0x00000004;
		const ACModTime = 0x00000008;
		const Extended = 0x80000000;
	}
}

#[derive(Clone, Debug, Default)]
pub struct FileAttributes {
	pub flags: FileAttrFlags,
	pub size: Option<u64>,
	pub uid: Option<u32>,
	pub gid: Option<u32>,
	pub permissions: Option<u32>,
	pub atime: Option<u32>,
	pub mtime: Option<u32>,
	// TODO:  Extended count, extended strings
}

impl Serialize for FileAttributes {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> /* {{{ */ {
		let mut field_count = 1;
		if(self.flags.contains(FileAttrFlags::Size)) {
			field_count += 1;
		}
		if(self.flags.contains(FileAttrFlags::UidGid)) {
			field_count += 2;
		}
		if(self.flags.contains(FileAttrFlags::Permissions)) {
			field_count += 1;
		}
		if(self.flags.contains(FileAttrFlags::ACModTime)) {
			field_count += 2;
		}
		// TODO:  Extended count and strings
		let mut state = serializer.serialize_struct("FileAttributes", field_count)?;
		state.serialize_field("flags", &(self.flags & !FileAttrFlags::Extended).bits())?;
		if(self.flags.contains(FileAttrFlags::Size)) {
			state.serialize_field("size", &self.size.unwrap_or(0))?;
		}
		if(self.flags.contains(FileAttrFlags::UidGid)) {
			state.serialize_field("uid", &self.uid.unwrap_or(0))?;
			state.serialize_field("gid", &self.gid.unwrap_or(0))?;
		}
		if(self.flags.contains(FileAttrFlags::Permissions)) {
			state.serialize_field("permissions", &self.permissions.unwrap_or(0))?;
		}
		if(self.flags.contains(FileAttrFlags::ACModTime)) {
			state.serialize_field("atime", &self.atime.unwrap_or(0))?;
			state.serialize_field("mtime", &self.mtime.unwrap_or(0))?;
		}
		state.end()
	} // }}}
}

impl FileAttributes {
	pub fn new() -> Self /* {{{ */ {
		Self{
			flags: FileAttrFlags::from_bits_truncate(0),
			..Default::default()
		}
	} // }}}

	pub fn set_size(&mut self, size: u64) /* {{{ */ {
		self.flags.set(FileAttrFlags::Size, true);
		self.size = Some(size);
	} // }}}

	pub fn get_uid_gid(&self) -> Option<(u32, u32)> /* {{{ */ {
		match self.flags.contains(FileAttrFlags::UidGid) {
			true => Some((self.uid.unwrap(), self.gid.unwrap())),
			false => None
		}
	} // }}}

	pub fn set_uid_gid(&mut self, uid: u32, gid: u32) /* {{{ */ {
		self.flags.set(FileAttrFlags::UidGid, true);
		self.uid = Some(uid);
		self.gid = Some(gid);
	} // }}}

	pub fn get_permissions(&self) -> Option<u32> /* {{{ */ {
		self.permissions
	} // }}}

	pub fn set_permissions(&mut self, permissions: u32) /* {{{ */ {
		self.flags.set(FileAttrFlags::Permissions, true);
		self.permissions = Some(permissions);
	} // }}}

	pub fn get_atime_mtime(&self) -> Option<(u32, u32)> /* {{{ */ {
		match self.flags.contains(FileAttrFlags::ACModTime) {
			true => Some((self.atime.unwrap(), self.mtime.unwrap())),
			false => None
		}
	} // }}}

	pub fn set_atime_mtime(&mut self, atime: u32, mtime: u32) /* {{{ */ {
		self.flags.set(FileAttrFlags::ACModTime, true);
		self.atime = Some(atime);
		self.mtime = Some(mtime);
	} // }}}

	pub fn parse(i: &[u8]) -> IResult<&[u8], Self> /* {{{ */ {
		let (mut i, flags) = be_u32(i)?;
		let mut attrs = Self{
			flags: FileAttrFlags::from_bits_truncate(flags),
			..Default::default()
		};
		if(attrs.flags.contains(FileAttrFlags::Size)) {
			let (i_inner, size) = be_u64(i)?;
			attrs.size = Some(size);
			i = i_inner;
		}
		if(attrs.flags.contains(FileAttrFlags::UidGid)) {
			let (i_inner, uid) = be_u32(i)?;
			let (i_inner, gid) = be_u32(i_inner)?;
			attrs.uid = Some(uid);
			attrs.gid = Some(gid);
			i = i_inner;
		}
		if(attrs.flags.contains(FileAttrFlags::Permissions)) {
			let (i_inner, permissions) = be_u32(i)?;
			attrs.permissions = Some(permissions);
			i = i_inner;
		}
		if(attrs.flags.contains(FileAttrFlags::ACModTime)) {
			let (i_inner, atime) = be_u32(i)?;
			let (i_inner, mtime) = be_u32(i_inner)?;
			attrs.atime = Some(atime);
			attrs.mtime = Some(mtime);
			i = i_inner
		}
		Ok((i, attrs))
	} // }}}

	pub fn binsize(&self) -> u32 /* {{{ */ {
		let mut size = 4;
		if(self.flags.contains(FileAttrFlags::Size)) {
			size += 8;
		}
		if(self.flags.contains(FileAttrFlags::UidGid)) {
			size += 8;
		}
		if(self.flags.contains(FileAttrFlags::Permissions)) {
			size += 4;
		}
		if(self.flags.contains(FileAttrFlags::ACModTime)) {
			size += 8;
		}
		size
	} // }}}
}

impl From<Metadata> for FileAttributes {
	fn from(metadata: Metadata) -> Self {
		let mut this = Self::new();
		this.set_size(metadata.size);
		this.set_uid_gid(metadata.uid, metadata.gid);
		this.set_permissions({
			let mut permissions = metadata.permissions;
			if(metadata.is_dir) {
				permissions = permissions | 0o40000;
			}
			permissions
		});
		this.set_atime_mtime(metadata.atime.timestamp() as u32, metadata.mtime.timestamp() as u32);
		this
	}
}

