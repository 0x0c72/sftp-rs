use std::collections::VecDeque;
use std::fs::Permissions;
use std::os::linux::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;

use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::Utc;

use tokio::fs::OpenOptions;
use tokio::fs::create_dir;
use tokio::fs::read_dir;
use tokio::fs::read_link;
use tokio::fs::remove_dir;
use tokio::fs::remove_file;
use tokio::fs::rename;
use tokio::fs::set_permissions;

use filetime::FileTime;
use filetime::set_file_times;

use sftp_protocol::common::Metadata;
use sftp_server::file::OpenFile;
use sftp_server::backend::Backend;
use sftp_server::backend::PathRef;
use sftp_server::backend::Result;

#[derive(Clone, Debug)]
pub struct Filesystem {
	root: PathBuf
}

lazy_static! {
	static ref ZEROTIME: DateTime<Utc> = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc);
}

async fn metadata(path: impl AsRef<Path>) -> Result<Metadata> {
	let meta = tokio::fs::metadata(&path).await?;
	let mut output = Metadata{
		path: path.as_ref().to_string_lossy().to_string(),
		size: meta.len(),
		is_dir: meta.is_dir(),
		is_file: meta.is_file(),
		link_target: match meta.file_type().is_symlink() {
			true => Some(read_link(path.as_ref()).await?.to_string_lossy().to_string()),
			false => None
		},
		uid: meta.st_uid(),
		gid: meta.st_gid(),
		// If we're on Windows, mode bits don't exist, so just lie.  TODO:  Figure out a decent way to synthesize on Windows.
		permissions: 0o755,
		atime: meta.accessed().map(|v| v.into()).unwrap_or(*ZEROTIME),
		mtime: meta.accessed().map(|v| v.into()).unwrap_or(*ZEROTIME)
	};
	if(cfg!(unix)) {
		output.permissions = meta.permissions().mode();
	}
	Ok(output)
}

#[async_trait]
impl Backend for Filesystem {
	async fn metadata(&self, path: impl PathRef + 'async_trait) -> Result<Metadata> {
		let path = self.full_normalize_path(path)?;
		metadata(&path).await
	}

	async fn list(&self, path: impl PathRef + 'async_trait) -> Result<VecDeque<Metadata>> {
		let path = self.full_normalize_path(path)?;
		let mut dir = read_dir(&path).await?;
		let mut result = VecDeque::new();
		while let Some(entry) = dir.next_entry().await? {
			let mut meta = metadata(entry.path()).await?;
			meta.path = entry.file_name().to_string_lossy().to_string();
			result.push_back(meta);
		}
		Ok(result)
	}

	async fn open(&self, path: impl PathRef + 'async_trait, read: bool, write: bool, append: bool, create: bool, truncate: bool, create_new: bool) -> Result<OpenFile> {
		let path = self.full_normalize_path(path)?;
		let fd = OpenOptions::new()
			.create(true)
			.read(read)
			.write(write)
			.append(append)
			.create(create)
			.truncate(truncate)
			.create_new(create_new)
			.open(&path)
			.await?;
		let metadata = metadata(&path).await?;
		Ok(OpenFile::new(metadata, fd))
	}

	async fn set_metadata(&self, path: impl PathRef + 'async_trait, uid_and_gid: Option<(u32, u32)>, permissions: Option<u32>, atime_and_mtime: Option<(u32, u32)>) -> Result<()> {
		let path = self.full_normalize_path(path)?;
		if let Some((uid, gid)) = uid_and_gid {
			if(cfg!(unix)) {
				let uid = nix::unistd::Uid::from_raw(uid);
				let gid = nix::unistd::Gid::from_raw(gid);
				tokio::task::block_in_place(|| nix::unistd::chown(&path, Some(uid), Some(gid)))?;
			} else {
				eprintln!("!!! Filesystem::set_metadata():  Can't set UID on non-Unix platforms");
			}
		}
		if let Some(permissions) = permissions {
			if(cfg!(unix)) {
				set_permissions(&path, Permissions::from_mode(permissions)).await?;
			} else {
				eprintln!("!!! Filesystem::set_metadata():  Can't set permissions on non-Unix platforms");
			}
		}
		if let Some((atime, mtime)) = atime_and_mtime {
			let atime = FileTime::from_unix_time(atime as i64, 0);
			let mtime = FileTime::from_unix_time(mtime as i64, 0);
			tokio::task::block_in_place(|| set_file_times(&path, atime, mtime))?;
		}
		Ok(())
	}

	async fn delete_file(&self, path: impl PathRef + 'async_trait) -> Result<()> {
		let path = self.full_normalize_path(path)?;
		remove_file(path).await?;
		Ok(())
	}

	async fn mkdir(&self, path: impl PathRef + 'async_trait) -> Result<()> {
		let path = self.full_normalize_path(path)?;
		create_dir(path).await?;
		Ok(())
	}

	async fn rmdir(&self, path: impl PathRef + 'async_trait) -> Result<()> {
		let path = self.full_normalize_path(path)?;
		remove_dir(path).await?;
		Ok(())
	}

	async fn rename(&self, from: impl PathRef + 'async_trait, to: impl PathRef + 'async_trait) -> Result<()> {
		let from = self.full_normalize_path(from)?;
		let to = self.full_normalize_path(to)?;
		// TODO:  This fails across mountpoints; when that happens, manually copy and delete the source
		rename(from, to).await?;
		Ok(())
	}
}

impl Filesystem {
	pub fn new(root: impl AsRef<Path>) -> Result<Self> {
		Ok(Self{
			root: root.as_ref().to_path_buf().canonicalize()?
		})
	}

	fn full_normalize_path(&self, path: impl AsRef<Path>) -> Result<PathBuf> {
		let result = self.root.join(self.normalize_path(path.as_ref())?);
		Ok(result)
	}

	fn canonicalize(&self, path: impl AsRef<Path>) -> Result<PathBuf> {
		Ok(self.full_normalize_path(path)?.canonicalize()?)
	}
}

