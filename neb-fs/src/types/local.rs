use std::sync::OnceLock;

use mutual::{RefGuard, RelaxedMutex, SharedData};
use tokio::{fs::OpenOptions, io::{AsyncReadExt, AsyncWriteExt}};

use crate::{BinaryFuture, EmptyFuture, FileSystem, ReadStream, ReadStreamFuture, TextFuture, WriteStream, WriteStreamFuture};

static LOCAL_FILE_SYSTEM_ONCE_LOCK: OnceLock<RelaxedMutex<Box<dyn FileSystem>>> = OnceLock::new();
pub fn local_file_system() -> RefGuard<Box<dyn FileSystem>> {
    let mutex = LOCAL_FILE_SYSTEM_ONCE_LOCK
        .get_or_init(|| RelaxedMutex::new(Box::new(LocalFileSystem)));
    return mutex.lock_ref();
}

/// [`FileSystem`] backed by the local disk via `tokio::fs`. Supports full
/// read/write and streaming.
pub struct LocalFileSystem;
impl FileSystem for LocalFileSystem {
    fn read_bytes(&self, path: String) -> BinaryFuture {
        Box::pin(async move {
            Ok(tokio::fs::read(path).await?)
        })
    }

    fn read_text(&self, path: String) -> TextFuture {
        Box::pin(async move {
            Ok(tokio::fs::read_to_string(path).await?)
        })
    }

    fn write_bytes(&self, path: String, bytes: Vec<u8>) -> EmptyFuture {
        Box::pin(async move {
            tokio::fs::write(path, bytes).await?;
            Ok(())
        })
    }

    fn write_text(&self, path: String, text: String) -> EmptyFuture {
        Box::pin(async move {
            tokio::fs::write(path, text).await?;
            Ok(())
        })
    }

    fn as_write_stream(&self, path: String) -> WriteStreamFuture {
        Box::pin(async move {
            let ws = LocalFileSystemWriteStream::new(&path).await?;
            Ok(Box::new(ws) as Box<dyn WriteStream>)
        })
    }

    fn as_read_stream(&self, path: String) -> ReadStreamFuture {
        Box::pin(async move {
            let ws = LocalFileSystemReadStream::new(&path).await?;
            Ok(Box::new(ws) as Box<dyn ReadStream>)
        })
    }
}

/// A [`LocalFileSystem`] file opened for chunked reads.
pub struct LocalFileSystemReadStream(RelaxedMutex<tokio::fs::File>);

impl LocalFileSystemReadStream {
    /// Opens `path` for reading.
    pub async fn new(path: &str) -> anyhow::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .open(path)
            .await?;

        Ok(Self(RelaxedMutex::new(file)))
    }
}

impl ReadStream for LocalFileSystemReadStream {
    fn stream_bytes(&mut self, count: usize) -> BinaryFuture {
        let mut file = self.0.lock_mut();
        Box::pin(async move {
            let mut buffer = vec![0_u8; count];
            file.read_exact(&mut buffer).await?;
            Ok(buffer)
        })
    }
}

/// A [`LocalFileSystem`] file opened for chunked appends, created if missing.
pub struct LocalFileSystemWriteStream(RelaxedMutex<tokio::fs::File>);

impl LocalFileSystemWriteStream {
    /// Opens `path` for appending, creating it if it doesn't exist.
    pub async fn new(path: &str) -> anyhow::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .await?;

        Ok(Self(RelaxedMutex::new(file)))
    }
}

impl WriteStream for LocalFileSystemWriteStream {
    fn append_bytes(&mut self, bytes: Vec<u8>) -> EmptyFuture {
        let mut file = self.0.lock_mut();
        Box::pin(async move {
            file.write(&bytes).await?;
            Ok(())
        })
    }
}

