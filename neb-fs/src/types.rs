use std::pin::Pin;

#[cfg(not(target_arch = "wasm32"))] pub mod local;
#[cfg(not(target_arch = "wasm32"))] pub mod web;
#[cfg(target_arch = "wasm32")] pub mod wasm;
#[cfg(target_arch = "wasm32")] pub mod node;

use anyhow::bail;
#[cfg(not(target_arch = "wasm32"))] pub use local::*;
#[cfg(not(target_arch = "wasm32"))] pub use web::*;
#[cfg(target_arch = "wasm32")] pub use wasm::*;
#[cfg(target_arch = "wasm32")] pub use node::*;

/// Pinned future returned by operations that produce no value on success.
pub type EmptyFuture<'a> = Pin<Box<dyn Future<Output = anyhow::Result<()>> + 'a>>;

/// Pinned future returned by operations that read raw bytes.
pub type BinaryFuture<'a> = Pin<Box<dyn Future<Output = anyhow::Result<Vec<u8>>> + 'a>>;

/// Pinned future returned by operations that read UTF-8 text.
pub type TextFuture<'a> = Pin<Box<dyn Future<Output = anyhow::Result<String>> + 'a>>;

/// Pinned future returned by operations that open a [`WriteStream`].
pub type WriteStreamFuture<'a> = Pin<Box<dyn Future<Output = anyhow::Result<Box<dyn WriteStream>>> + 'a>>;

/// Pinned future returned by operations that open a [`ReadStream`].
pub type ReadStreamFuture<'a> = Pin<Box<dyn Future<Output = anyhow::Result<Box<dyn ReadStream>>> + 'a>>;

/// A backend for reading and, optionally, writing files by path. Only the
/// read methods are required; write and streaming methods default to
/// returning an error for backends that don't support them (e.g. the
/// HTTP-backed `WebFileSystem` is read-only).
pub trait FileSystem: Send + Sync {
    fn read_bytes<'a>(&self, path: &'a str) -> BinaryFuture<'a>;
    fn read_text<'a>(&self, path: &'a str) -> TextFuture<'a>;
    
    #[allow(unused)]
    fn write_bytes<'a>(&self, path: &'a str, bytes: Vec<u8>) -> EmptyFuture<'a> { 
        Box::pin(async move { bail!("Immutable file system") }) 
    }

    #[allow(unused)]
    fn write_text<'a>(&self, path: &'a str, text: String) -> EmptyFuture<'a> { 
        Box::pin(async move { bail!("Immutable file system") }) 
    }

    #[allow(unused)]
    fn as_write_stream<'a>(&self, path: &'a str) -> WriteStreamFuture<'a> {
        Box::pin(async move { bail!("Write stream not supported") }) 
    }
    
    #[allow(unused)]
    fn as_read_stream<'a>(&self, path: &'a str) -> ReadStreamFuture<'a> { 
        Box::pin(async move { bail!("Read stream not supported") }) 
    }
}

/// A file opened for appending bytes in chunks.
pub trait WriteStream: Send + Sync {
    fn append_bytes<'a>(&'a mut self, bytes: Vec<u8>) -> EmptyFuture<'a>;
}

/// A file opened for reading bytes in fixed-size chunks.
pub trait ReadStream: Send + Sync {
    fn stream_bytes<'a>(&'a mut self, count: usize) -> BinaryFuture<'a>;
}
