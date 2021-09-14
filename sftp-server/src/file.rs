use core::task::Context;
use core::task::Poll;
use core::task::Poll::Ready;

use std::fmt;
use std::pin::Pin;

use tokio::io::AsyncRead;
use tokio::io::AsyncSeek;
use tokio::io::AsyncWrite;
use tokio::io::Error;
use tokio::io::SeekFrom;

use sftp_protocol::common::Metadata;

pub trait File: AsyncRead + AsyncSeek + AsyncWrite + Send + Sync + Unpin + fmt::Debug {}
impl<T> File for T where T: AsyncRead + AsyncSeek + AsyncWrite + Send + Sync + Unpin + fmt::Debug {}

pub struct OpenFile {
	pub metadata: Metadata,
	pub pos: u64,
	pub fd: Pin<Box<dyn File>>,
}

impl OpenFile {
	pub fn new(metadata: Metadata, stream: impl File + 'static) -> Self {
		Self{
			metadata: metadata,
			pos: 0,
			fd: Box::pin(stream)
		}
	}
}

impl fmt::Debug for OpenFile {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("OpenFile")
			.field("pos", &self.pos)
			.field("fd", &self.fd)
			.finish()
	}
}

#[derive(thiserror::Error, Debug)]
pub enum IOError {
	#[error("not open for reading")]
	NoRead,
	#[error("not open for writing")]
	NoWrite
}

impl AsyncRead for OpenFile {
	fn poll_read(mut self: Pin<&mut Self>, ctx: &mut Context<'_>, buf: &mut [u8]) -> Poll<Result<usize, Error>> {
		let result = self.fd.as_mut().poll_read(ctx, buf);
		if let Ready(Ok(count)) = result {
			self.pos += count as u64;
		}
		result
	}
}

impl AsyncWrite for OpenFile {
	fn poll_write(mut self: Pin<&mut Self>, ctx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, Error>> {
		let result = self.fd.as_mut().poll_write(ctx, buf);
		if let Ready(Ok(count)) = result {
			self.pos += count as u64;
		}
		result
	}

	fn poll_flush(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Result<(), Error>> {
		self.fd.as_mut().poll_flush(ctx)
	}

	fn poll_shutdown(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Result<(), Error>> {
		self.fd.as_mut().poll_shutdown(ctx)
	}
}

impl AsyncSeek for OpenFile {
	fn start_seek(mut self: Pin<&mut Self>, ctx: &mut Context<'_>, position: SeekFrom) -> Poll<Result<(), Error>> {
		// TODO:  Short circuit if position is SeekFrom::Start(pos) where pos == self.pos
		self.fd.as_mut().start_seek(ctx, position)
	}

	fn poll_complete(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Result<u64, Error>> {
		let result = self.fd.as_mut().poll_complete(ctx);
		if let Ready(Ok(pos)) = result {
			self.pos = pos;
		}
		result
	}
}

