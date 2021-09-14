use chrono::DateTime;
use chrono::Utc;

#[derive(Clone, Debug)]
pub struct Metadata {
	pub path: String,
	pub size: u64,
	pub is_dir: bool,
	pub is_file: bool,
	pub link_target: Option<String>,
	pub uid: u32,
	pub gid: u32,
	pub permissions: u32,
	pub atime: DateTime<Utc>,
	pub mtime: DateTime<Utc>
}

impl Metadata {
	pub fn new(path: &str, size: u64, is_dir: bool, is_file: bool, link_target: Option<&str>, uid: u32, gid: u32, permissions: u32, atime: impl Into<DateTime<Utc>>, mtime: impl Into<DateTime<Utc>>) -> Self {
		Self{
			path: path.to_string(),
			size: size,
			is_dir: is_dir,
			is_file: is_file,
			link_target: link_target.map(String::from),
			uid: uid,
			gid: gid,
			permissions: permissions,
			atime: atime.into(),
			mtime: mtime.into()
		}
	}
}

