use crate::{BinaryFuture, EmptyFuture, FileSystem, TextFuture};

pub struct File<'a> {
    file_system: &'a dyn FileSystem,
    path: &'a str
}

impl <'a> File<'a> {
    pub fn open(file_system: &'a dyn FileSystem, path: impl Into<&'a str>) -> Self {
        Self { file_system, path: path.into() }
    }

    pub fn read_bytes(&self) -> BinaryFuture<'a> {
        self.file_system.read_bytes(self.path)
    }

    pub fn read_text(&self) -> TextFuture<'a> {
        self.file_system.read_text(self.path)
    }

    pub fn write_bytes(&self, bytes: Vec<u8>) -> EmptyFuture<'a> {
        self.file_system.write_bytes(self.path, bytes)
    }

    pub fn write_text(&self, text: impl Into<String>) -> EmptyFuture<'a> {
        self.file_system.write_text(self.path, text.into())
    }
}
