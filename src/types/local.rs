// ponytail: std::fs blocks the calling task (tokio::fs only moved it to a blocking
// pool anyway). Bring back spawn_blocking if a hot async path starts stalling.

use crate::{BinaryFuture, EmptyFuture, FileSystem, TextFuture};

pub static LOCAL_FILE_SYSTEM: &'static LocalFileSystem = &LocalFileSystem;

pub struct LocalFileSystem;
impl FileSystem for LocalFileSystem {
    fn read_bytes<'a>(&self, path: &'a str) -> BinaryFuture<'a> {
        Box::pin(async move {
            Ok(std::fs::read(path)?)
        })
    }

    fn read_text<'a>(&self, path: &'a str) -> TextFuture<'a> {
        Box::pin(async move {
            Ok(std::fs::read_to_string(path)?)
        })
    }

    fn write_bytes<'a>(&self, path: &'a str, bytes: Vec<u8>) -> EmptyFuture<'a> {
        Box::pin(async move {
            std::fs::write(path, bytes)?;
            Ok(())
        })
    }

    fn write_text<'a>(&self, path: &'a str, text: String) -> EmptyFuture<'a> {
        Box::pin(async move {
            std::fs::write(path, text)?;
            Ok(())
        })
    }
}

