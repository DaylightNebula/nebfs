use crate::{BinaryFuture, EmptyFuture, FileSystem, TextFuture};

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
}

