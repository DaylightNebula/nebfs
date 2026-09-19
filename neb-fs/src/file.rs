use mutual::RefGuard;

use crate::{BinaryFuture, EmptyFuture, FileSystem, ReadStreamFuture, TextFuture, WriteStreamFuture};

/// A path bound to a [`FileSystem`], so reads, writes, and streams can be
/// called without repeating the backend and path at each call site.
pub struct VirtualFile {
    file_system: RefGuard<Box<dyn FileSystem>>,
    path: String
}

impl VirtualFile {
    /// Binds `path` to `file_system` for subsequent operations.
    pub fn open(file_system: RefGuard<Box<dyn FileSystem>>, path: impl Into<String>) -> Self {
        Self { file_system, path: path.into() }
    }

    /// Reads the whole file as bytes.
    pub fn read_bytes(&self) -> BinaryFuture {
        self.file_system.read_bytes(self.path.clone())
    }

    /// Reads the whole file as UTF-8 text.
    pub fn read_text(&self) -> TextFuture {
        self.file_system.read_text(self.path.clone())
    }

    /// Overwrites the file with `bytes`.
    pub fn write_bytes(&self, bytes: Vec<u8>) -> EmptyFuture {
        self.file_system.write_bytes(self.path.clone(), bytes)
    }

    /// Overwrites the file with `text`.
    pub fn write_text(&self, text: impl Into<String>) -> EmptyFuture {
        self.file_system.write_text(self.path.clone(), text.into())
    }

    /// Opens the file for reading in fixed-size chunks.
    pub fn as_read_stream(&self) -> ReadStreamFuture {
        self.file_system.as_read_stream(self.path.clone())
    }

    /// Opens the file for appending in chunks, creating it if needed.
    pub fn as_write_stream(&self) -> WriteStreamFuture {
        self.file_system.as_write_stream(self.path.clone())
    }
}
