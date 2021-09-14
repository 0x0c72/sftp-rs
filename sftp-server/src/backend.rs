use std::collections::VecDeque;
use std::path::Path;
use std::path::PathBuf;

use lexiclean::Lexiclean;

use sftp_protocol::Error;
use sftp_protocol::common::Metadata;
use super::file::OpenFile;

pub type Result<T> = std::result::Result<T, Error>;
pub trait PathRef: AsRef<Path> + Send {}
impl<T> PathRef for T where T: AsRef<Path> + Send {}

#[async_trait]
pub trait Backend : Clone {
	async fn metadata(&self, path: impl PathRef + 'async_trait) -> Result<Metadata>;
	async fn list(&self, path: impl PathRef + 'async_trait) -> Result<VecDeque<Metadata>>;
	async fn open(&self, path: impl PathRef + 'async_trait, read: bool, write: bool, append: bool, create: bool, truncate: bool, create_new: bool) -> Result<OpenFile>;
	async fn set_metadata(&self, path: impl PathRef + 'async_trait, uid_and_gid: Option<(u32, u32)>, permissions: Option<u32>, atime_and_mtime: Option<(u32, u32)>) -> Result<()>;
	async fn delete_file(&self, path: impl PathRef + 'async_trait) -> Result<()>;
	async fn mkdir(&self, path: impl PathRef + 'async_trait) -> Result<()>;
	async fn rmdir(&self, path: impl PathRef + 'async_trait) -> Result<()>;
	async fn rename(&self, from: impl PathRef + 'async_trait, to: impl PathRef + 'async_trait) -> Result<()>;

	fn normalize_path(&self, path: impl AsRef<Path>) -> Result<PathBuf> {
		let path = path.as_ref().lexiclean();
		if let Some(first) = path.ancestors().next() {
			if(first == PathBuf::from("..")) {
				return Err(Error::InvalidPath);
			}
		}
		Ok(path)
	}
}

