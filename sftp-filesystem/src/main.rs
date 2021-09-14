#![allow(unused_parens)]
#![allow(non_upper_case_globals)]
#[macro_use] extern crate async_trait;
#[macro_use] extern crate lazy_static;

use std::fs::OpenOptions;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use tokio::fs::create_dir_all;
use tokio::task::block_in_place;

use anyhow::Error;
use envconfig::Envconfig;

#[cfg(feature = "standalone")]
use thrussh_keys::key::KeyPair;
#[cfg(feature = "standalone")]
use thrussh_keys::PublicKeyBase64;

use sftp_server::Server;

mod filesystem;
use filesystem::Filesystem;

#[derive(Envconfig)]
struct Config {
	#[envconfig(from = "CONFIG_DIR", default = "/tmp/sftp/config")]
	pub config_dir: PathBuf,
	#[envconfig(from = "DATA_DIR", default = "/tmp/sftp/data")]
	pub data_dir: PathBuf,
	#[envconfig(from = "SSH_PORT", default = "2222")]
	pub port: u16
}

#[cfg(feature = "standalone")]
async fn load_or_create_keypair(path_private: impl AsRef<Path>, path_public: impl AsRef<Path>, passphrase: Option<&[u8]>) -> Result<KeyPair, Error> /* {{{ */ {
	block_in_place(move || {
		match thrussh_keys::load_secret_key(&path_private, passphrase) {
			Ok(v) => {
				eprintln!("--- loaded keypair from {}", path_private.as_ref().to_str().unwrap());
				Ok(v)
			},
			Err(_) => {
				let key = thrussh_keys::key::KeyPair::generate_ed25519().unwrap();
				let f = OpenOptions::new().create(true).truncate(true).write(true).read(false).open(&path_private)?;
				thrussh_keys::encode_pkcs8_pem(&key, f)?;
				eprintln!("--- created ED25519 keypair and wrote it to {}", path_private.as_ref().to_str().unwrap());
				let f = OpenOptions::new().create(true).truncate(true).write(true).read(false).open(&path_public)?;
				thrussh_keys::write_public_key_base64(f, &key.clone_public_key())?;
				eprintln!("--- wrote public key to {}", path_public.as_ref().to_str().unwrap());
				Ok(key)
			}
		}
	})
} // }}}

#[tokio::main]
async fn main() {
	env_logger::init();
	let config = Config::init().unwrap();
	create_dir_all(&config.config_dir).await.unwrap();
	create_dir_all(&config.data_dir).await.unwrap();

	let backend = Filesystem::new(&config.data_dir).unwrap();
	let mut server = Server::new(backend, 0);

	#[cfg(feature = "standalone")]
	{
		let mut ssh_config = thrussh::server::Config::default();
		ssh_config.connection_timeout = Some(Duration::from_secs(60));
		ssh_config.auth_rejection_time = Duration::from_secs(1);
		ssh_config.keys.push(load_or_create_keypair(config.config_dir.join("test.server.key"), config.config_dir.join("test.server.key.pub"), None).await.unwrap());
		let ssh_config = Arc::new(ssh_config);

		thrussh::server::run(ssh_config, &format!("0.0.0.0:{}", config.port), server).await.unwrap();
	}
	#[cfg(feature = "legacy")]
	server.run().await.unwrap();
}

