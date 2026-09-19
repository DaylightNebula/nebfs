use tokio::{fs::OpenOptions, io::{AsyncReadExt, AsyncWriteExt}};

use crate::{BinaryFuture, EmptyFuture, FileSystem, ReadStream, ReadStreamFuture, TextFuture, WriteStream, WriteStreamFuture};

pub static LOCAL_FILE_SYSTEM: &'static LocalFileSystem = &LocalFileSystem;

pub struct LocalFileSystem;
impl FileSystem for LocalFileSystem {
    fn read_bytes<'a>(&self, path: &'a str) -> BinaryFuture<'a> {
        Box::pin(async move {
            Ok(tokio::fs::read(path).await?)
        })
    }

    fn read_text<'a>(&self, path: &'a str) -> TextFuture<'a> {
        Box::pin(async move {
            Ok(tokio::fs::read_to_string(path).await?)
        })
    }

    fn write_bytes<'a>(&self, path: &'a str, bytes: Vec<u8>) -> EmptyFuture<'a> {
        Box::pin(async move {
            tokio::fs::write(path, bytes).await?;
            Ok(())
        })
    }

    fn write_text<'a>(&self, path: &'a str, text: String) -> EmptyFuture<'a> {
        Box::pin(async move {
            tokio::fs::write(path, text).await?;
            Ok(())
        })
    }

    fn as_write_stream<'a>(&self, path: &'a str) -> WriteStreamFuture<'a> {
        Box::pin(async move {
            let ws = LocalFileSystemWriteStream::new(path).await?;
            Ok(Box::new(ws) as Box<dyn WriteStream>)
        })
    }

    fn as_read_stream<'a>(&self, path: &'a str) -> ReadStreamFuture<'a> {
        Box::pin(async move {
            let ws = LocalFileSystemReadStream::new(path).await?;
            Ok(Box::new(ws) as Box<dyn ReadStream>)
        })
    }
}

pub struct LocalFileSystemReadStream(tokio::fs::File);

impl LocalFileSystemReadStream {
    pub async fn new(path: &str) -> anyhow::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .open(path)
            .await?;

        Ok(Self(file))
    }
}

impl ReadStream for LocalFileSystemReadStream {
    fn stream_bytes<'a>(&'a mut self, count: usize) -> BinaryFuture<'a> {
        Box::pin(async move {
            let mut buffer = vec![0_u8; count];
            self.0.read_exact(&mut buffer).await?;
            Ok(buffer)
        })
    }
}

pub struct LocalFileSystemWriteStream(tokio::fs::File);

impl LocalFileSystemWriteStream {
    pub async fn new(path: &str) -> anyhow::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .await?;

        Ok(Self(file))
    }
}

impl WriteStream for LocalFileSystemWriteStream {
    fn append_bytes<'a>(&'a mut self, bytes: Vec<u8>) -> EmptyFuture<'a> {
        Box::pin(async move {
            self.0.write(&bytes).await?;
            Ok(())
        })
    }
}

