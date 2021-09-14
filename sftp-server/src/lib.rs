#![allow(unused_parens)]
#![allow(non_upper_case_globals)]
#[macro_use] extern crate async_trait;
#[macro_use] extern crate lazy_static;

use std::collections::HashMap;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use futures::executor::block_on;
use futures::future::Ready;
use futures::future::ready;

use tokio::io::AsyncReadExt;
use tokio::io::AsyncSeekExt;
use tokio::io::AsyncWriteExt;
use tokio::io::SeekFrom;
use tokio::time::delay_for;
use tokio::time::Duration;

use anyhow::Error;

use bincode::Options;

#[cfg(feature = "standalone")]
use thrussh::{
	ChannelId,
	CryptoVec,
	server::{
		Auth,
		Handle,
		Handler,
		Response,
		Session
	}
};

use uuid::Uuid;

use sftp_protocol::stream::packet;
use sftp_protocol::stream::packet::name::File;
use sftp_protocol::stream::packet::open::OpenFlags;
use sftp_protocol::stream::packet::PayloadTrait;
use sftp_protocol::stream::packet::status::StatusType;
use sftp_protocol::Packet;
use sftp_protocol::Payload;

pub mod backend;
use backend::Backend;
pub mod file;
use file::OpenFile;

#[derive(Clone, Debug)]
pub struct PartialPacket {
	target_len: Option<u32>,
	buffer: Vec<u8>
}

impl PartialPacket {
	fn new() -> Self {
		Self{
			target_len: None,
			buffer: Vec::new()
		}
	}
}

#[derive(Clone)]
pub struct Server<B: Backend + Send> {
	backend: Arc<Mutex<B>>,

	#[cfg(feature = "standalone")]
	pub clients: Arc<Mutex<HashMap<(usize, ChannelId), Handle>>>,
	pub id: usize,

	// In order to support large directories without blowing up, this may end up needing to hold a Stream<Item=File> instead of VecDeque<File>; for now this is fine.
	open_dirs: Arc<Mutex<HashMap<Uuid, (VecDeque<File>, usize)>>>,
	open_files: Arc<Mutex<HashMap<Uuid, OpenFile>>>,
	#[cfg(feature = "standalone")]
	partial_packets: HashMap<ChannelId, PartialPacket>,
}

fn parse_packet(data: &[u8], partial_packet: &mut PartialPacket) -> Result<Option<Packet>, Error> /* {{{ */ {
	let packet;
	if let Some(target_len) = partial_packet.target_len {
		partial_packet.buffer.extend_from_slice(data);
		if(partial_packet.buffer.len() < target_len as usize) {
			return Ok(None);
		}
		packet = match Packet::parse(&partial_packet.buffer) {
			Ok((_, v)) => v,
			Err(e) => return Err(Error::msg(format!("Failed to parse packet:  {}", e)))
		};
	} else {
		packet = match Packet::parse(&data) {
			Ok((_, v)) => v,
			Err(e) => return Err(Error::msg(format!("Failed to parse packet:  {}", e)))
		};
		if(data.len() < packet.header.length as usize) {
			partial_packet.target_len = Some(packet.header.length);
			partial_packet.buffer = Vec::from(data);
			return Ok(None);
		}
	}
	Ok(Some(packet))
} // }}}

impl<B: Backend + Send> Server<B> {
	pub fn new(backend: B, id: usize) -> Self /* {{{ */ {
		Self{
			backend: Arc::new(Mutex::new(backend)),
			#[cfg(feature = "standalone")]
			clients: Arc::new(Mutex::new(HashMap::new())),
			id: id,
			open_dirs: Arc::new(Mutex::new(HashMap::new())),
			open_files: Arc::new(Mutex::new(HashMap::new())),
			#[cfg(feature = "standalone")]
			partial_packets: HashMap::new(),
		}
	} // }}}

	async fn process_request(&self, input: Packet) -> Result<Packet, Error> /* {{{ */ {
		let output = match input.payload {
			Payload::Init(_) => Payload::version(3, vec![]).into_packet(),
			Payload::Version(_) => unreachable!(),
			Payload::Open(r) => /* {{{ */ {
				let path = PathBuf::from(r.path);
				let result = self.backend.lock().unwrap().open(
					&path,
					r.pflags.contains(OpenFlags::Read),
					r.pflags.contains(OpenFlags::Write),
					r.pflags.contains(OpenFlags::Append),
					r.pflags.contains(OpenFlags::Create),
					r.pflags.contains(OpenFlags::Truncate),
					r.pflags.contains(OpenFlags::Exclude)
				).await;
				let response = match result {
					Ok(v) => {
						let response = Payload::handle(r.id);
						let mut state = self.open_files.lock().unwrap();
						state.insert(response.handle.clone(), v);
						Payload::Handle(response)
					},
					Err(e) => {
						eprintln!("!!! Failed to open file: {:?}", e);
						Payload::status(r.id, StatusType::Failure, format!("Failed to open file: {}", e))
					}
				};
				response.into_packet()
			}, // }}}
			Payload::Close(r) => /* {{{ */ {
				let mut files = self.open_files.lock().unwrap();
				let response = match files.remove(&r.handle) {
					Some(_) => Payload::status(r.id, StatusType::OK, "OK"),
					None => {
						let mut dirs = self.open_dirs.lock().unwrap();
						match dirs.remove(&r.handle) {
							Some(_) => Payload::status(r.id, StatusType::OK, "OK"),
							None => Payload::status(r.id, StatusType::NoSuchFile, format!("Handle {} does not exist", &r.handle))
						}
					}
				};
				response.into_packet()
			}, // }}}
			Payload::Read(r) => /* {{{ */ {
				let mut state = self.open_files.lock().unwrap();
				let response = match state.get_mut(&r.handle) {
					Some(ref mut file) => {
						let mut packet = Payload::data_with_size(r.id, r.len);
						file.seek(SeekFrom::Start(r.offset)).await?;
						let count = file.read(&mut packet.data).await?;
						if(count == 0) {
							Payload::status(r.id, StatusType::EOF, "EOF")
						} else {
							packet.data.truncate(count);
							Payload::Data(packet)
						}
					},
					None => Payload::status(r.id, StatusType::NoSuchFile, "No such file")
				};
				response.into_packet()
			}, // }}}
			Payload::Write(r) => /* {{{ */ {
				let mut state = self.open_files.lock().unwrap();
				let response = match state.get_mut(&r.handle) {
					Some(ref mut file) => {
						// TODO:  When attempting to seek past the end of the (existing), zerofill the gap
						file.seek(SeekFrom::Start(r.offset)).await?;
						let mut written = 0;
						while(written < r.data.len()) {
							let count = file.write(&r.data[written..]).await?;
							written += count;
						}
						if(written < r.data.len()) {
							Payload::status(r.id, StatusType::EOF, "EOF")
						} else {
							Payload::status(r.id, StatusType::OK, "OK")
						}
					},
					None => Payload::status(r.id, StatusType::NoSuchFile, "No such file")
				};
				response.into_packet()
			}, // }}}
			Payload::Lstat(r) => /* {{{ */ {
				// TODO:  Don't follow symlinks
				let mut attrs = Payload::attrs(r.id);
				attrs.attrs = self.backend.lock().unwrap().metadata(&r.path).await?.into();
				attrs.into_packet()
			}, // }}}
			Payload::Fstat(r) => /* {{{ */ {
				let open_files = self.open_files.lock().unwrap();
				let response = match open_files.get(&r.handle) {
					Some(v) => {
						let mut attrs = Payload::attrs(r.id);
						attrs.attrs = v.metadata.clone().into();
						Payload::Attrs(attrs)
					},
					None => Payload::status(r.id, StatusType::NoSuchFile, "Handle not found")
				};
				response.into_packet()
			}, // }}}
			Payload::SetStat(r) => /* {{{ */ {
				let response = match self.backend.lock().unwrap().set_metadata(&r.path, r.attrs.get_uid_gid(), r.attrs.get_permissions(), r.attrs.get_atime_mtime()).await {
					Ok(_) => Payload::status(r.id, StatusType::OK, "OK"),
					Err(e) => {
						eprintln!("!!! Failed to set metadata on {}:  {:?}", &r.path, e);
						Payload::status(r.id, StatusType::Failure, format!("Failed to set metadata: {}", e))
					}
				};
				response.into_packet()
			}, // }}}
			Payload::FSetStat(r) => /* {{{ */ {
				let open_files = self.open_files.lock().unwrap();
				let response = match open_files.get(&r.handle) {
					Some(v) => match self.backend.lock().unwrap().set_metadata(&v.metadata.path, r.attrs.get_uid_gid(), r.attrs.get_permissions(), r.attrs.get_atime_mtime()).await {
						Ok(_) => Payload::status(r.id, StatusType::OK, "OK"),
						Err(e) => {
							eprintln!("!!! Failed to set metadata on {}:  {}", &v.metadata.path, e);
							Payload::status(r.id, StatusType::Failure, format!("Failed to set metadata: {}", e))
						}
					},
					None => Payload::status(r.id, StatusType::NoSuchFile, "Handle not found")
				};
				response.into_packet()
			}, // }}}
			Payload::OpenDir(r) => /* {{{ */ {
				let response = Payload::handle(r.id);
				let contents = self.backend.lock().unwrap().list(&r.path).await?;
				self.open_dirs.lock().unwrap().insert(
					response.handle.clone(), (
						contents.into_iter().map(|f| File{
							longname: PathBuf::from(&r.path).join(&f.path).to_string_lossy().to_string(),
							filename: f.path.clone(),
							attrs: f.into()
						}).collect(),
						0
					)
				);
				response.into_packet()
			}, // }}}
			Payload::ReadDir(r) => /* {{{ */ {
				let mut state = self.open_dirs.lock().unwrap();
				match state.get_mut(&r.handle) {
					Some((ref mut files, ref mut index)) => {
						// TODO:  I would imagine that there's a limit on the response size.  We'll end up needing to chunk.
						if(*index >= files.len()) {
							Payload::status(r.id, StatusType::EOF, "EOF").into_packet()
						} else {
							let mut payload = Payload::name(r.id);
							if(files.len() > 0) {
								payload.files = files.clone().into();
							}
							*index += payload.files.len();
							payload.into_packet()
						}
					},
					None => Payload::status(r.id, StatusType::EOF, "EOF").into_packet()
				}
			}, // }}}
			Payload::Remove(r) => /* {{{ */ {
				let response = match self.backend.lock().unwrap().delete_file(&r.path).await {
					Ok(_) => Payload::status(r.id, StatusType::OK, "OK"),
					Err(e) => Payload::status(r.id, StatusType::Failure, format!("Failed to delete file: {}", e))
				};
				response.into_packet()
			}, // }}}
			Payload::MkDir(r) => /* {{{ */ {
				let response = match self.backend.lock().unwrap().mkdir(&r.path).await {
					Ok(_) => Payload::status(r.id, StatusType::OK, "OK"),
					Err(e) => Payload::status(r.id, StatusType::Failure, format!("Failed to create directory: {}", e))
				};
				response.into_packet()
			}, // }}}
			Payload::RmDir(r) => /* {{{ */ {
				let response = match self.backend.lock().unwrap().rmdir(&r.path).await {
					Ok(_) => Payload::status(r.id, StatusType::OK, "OK"),
					Err(e) => Payload::status(r.id, StatusType::Failure, format!("Failed to delete directory: {}", e))
				};
				response.into_packet()
			}, // }}}
			Payload::RealPath(r) => /* {{{ */ {
				let mut name = packet::name::Name::new(r.id);
				let backend = self.backend.lock().unwrap();
				let normalized = backend.normalize_path(&r.path)?.to_string_lossy().to_string();
				name.append_file(
					&normalized,
					&normalized,
					backend.metadata(&r.path).await?.into()
				);
				name.into_packet()
			}, // }}}
			Payload::Stat(r) => /* {{{ */ {
				// TODO:  Follow symlinks
				let mut attrs = Payload::attrs(r.id);
				attrs.attrs = self.backend.lock().unwrap().metadata(&r.path).await?.into();
				attrs.into_packet()
			}, // }}}
			Payload::Rename(r) => /* {{{ */ {
				let response = match self.backend.lock().unwrap().rename(&r.oldpath, &r.newpath).await {
					Ok(_) => Payload::status(r.id, StatusType::OK, "OK"),
					Err(e) => Payload::status(r.id, StatusType::Failure, format!("Failed to rename: {}", e))
				};
				response.into_packet()
			}, // }}}
			Payload::ReadLink(_) => unimplemented!(),
			Payload::Symlink(_) => unimplemented!(),
			Payload::Status(_) => unreachable!(),
			Payload::Handle(_) => unreachable!(),
			Payload::Data(_) => unreachable!(),
			Payload::Name(_) => unreachable!(),
			Payload::Attrs(_) => unreachable!(),
			Payload::Extended(_) => unimplemented!(),
			Payload::ExtendedReply(_) => unreachable!()
		};
		Ok(output)
	} // }}}

	async fn process_packet(&mut self, packet: Packet) -> Result<Option<Vec<u8>>, Error> /* {{{ */ {
		let se = bincode::DefaultOptions::new().with_big_endian().with_fixint_encoding();
		let response = self.process_request(packet).await?;
		let response_bytes = se.serialize(&response)?;
		Ok(Some(response_bytes))
	} // }}}

	#[cfg(feature = "legacy")]
	pub async fn run(&mut self) -> Result<(), Error> /* {{{ */ {
		let mut buf = [0u8; 8192];
		let mut partial_packet = PartialPacket::new();
		let mut stdin = tokio::io::stdin();
		let mut stdout = tokio::io::stdout();
		// TODO:  This needs a way to be terminated
		loop {
			let count = stdin.read(&mut buf).await?;
			if(count == 0) {
				delay_for(Duration::from_millis(10)).await;
				continue;
			}
			let packet = match parse_packet(&buf[0..count], &mut partial_packet) {
				Ok(Some(v)) => v,
				Ok(None) => continue,
				Err(e) => {
					eprintln!("!!! run():  Failed to parse packet:  {:?}", e);
					continue;
				}
			};
			let response = match block_on(self.process_packet(packet)) {
				Ok(Some(v)) => v,
				Ok(None) => continue,
				Err(e) => {
					eprintln!("!!! run():  Failed to process packet:  {:?}", e);
					continue;
				}
			};
			stdout.write(&response).await?;
			stdout.flush().await?;
		}
		Ok(())
	} // }}}
}

#[cfg(feature = "standalone")]
impl<B: Backend + Send> thrussh::server::Server for Server<B> {
	type Handler = Self;
	fn new(&mut self, _: Option<std::net::SocketAddr>) -> Self /* {{{ */ {
		let s = self.clone();
		self.id += 1;
		s
	} // }}}
}

#[cfg(feature = "standalone")]
impl<B: Backend + Send> Handler for Server<B> {
	type FutureAuth = Ready<Result<(Self, Auth), Error>>;
	type FutureUnit = Ready<Result<(Self, Session), Error>>;
	type FutureBool = Ready<Result<(Self, Session, bool), Error>>;

	fn finished_auth(self, auth: Auth) -> Self::FutureAuth /* {{{ */ {
		ready(Ok((self, auth)))
	} // }}}

	fn finished_bool(self, result: bool, session: Session) -> Self::FutureBool /* {{{ */ {
		ready(Ok((self, session, result)))
	} // }}}

	fn finished(self, session: Session) -> Self::FutureUnit /* {{{ */ {
		ready(Ok((self, session)))
	} // }}}

	fn channel_open_session(self, channel: ChannelId, session: Session) -> Self::FutureUnit /* {{{ */ {
		{
			let mut clients = self.clients.lock().unwrap();
			clients.insert((self.id, channel), session.handle());
		}
		self.finished(session)
	} // }}}

	fn auth_publickey(self, _: &str, _: &thrussh_keys::key::PublicKey) -> Self::FutureAuth /* {{{ */ {
		// TODO:  Actually validate authenticaiton.
		eprintln!("auth key success");
		self.finished_auth(Auth::Accept)
	} // }}}

	fn auth_keyboard_interactive(self, user: &str, submethods: &str, response: Option<Response>) -> Self::FutureAuth /* {{{ */ {
		// TODO:  Actually validate authentication.
		eprintln!("auth_keyboard_interactive('{}', '{}', {:?})", user, submethods, response);
		eprintln!("auth int success");
		self.finished_auth(Auth::Accept)
	} // }}}

	fn data(mut self, channel: ChannelId, data: &[u8], mut session: Session) -> Self::FutureUnit /* {{{ */ {
		let mut partial_packet = self.partial_packets.entry(channel).or_insert_with(|| PartialPacket::new());
		let packet = match parse_packet(data, &mut partial_packet) {
			Ok(Some(v)) => v,
			Ok(None) => return self.finished(session),
			Err(e) => {
				eprintln!("!!! data():  Failed to parse packet in channel {:?}:  {:?}", channel, e);
				return ready(Err(e));
			}
		};
		let response = match block_on(self.process_packet(packet)) {
			Ok(Some(v)) => v,
			Ok(None) => return self.finished(session),
			Err(e) => {
				eprintln!("!!! data():  Failed to process packet in channel {:?}:  {:?}", channel, e);
				return ready(Err(e));
			}
		};
		session.data(channel, response.into());
		self.finished(session)
	} // }}}

	fn subsystem_request(self, channel: ChannelId, name: &str, session: Session) -> Self::FutureUnit /* {{{ */ {
		// TODO:  We should keep state at the Server level for whether or not
		//    the SFTP subsystem has indeed been requested for a given channel
		//    ID.
		self.finished(session)
	} // }}}
}

