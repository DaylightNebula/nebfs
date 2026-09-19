use std::pin::Pin;

use anyhow::bail;

#[cfg(not(target_arch = "wasm32"))] pub mod local;
#[cfg(not(target_arch = "wasm32"))] pub mod web;
#[cfg(target_arch = "wasm32")] pub mod wasm;
#[cfg(target_arch = "wasm32")] pub mod node;

#[cfg(not(target_arch = "wasm32"))] pub use local::*;
#[cfg(not(target_arch = "wasm32"))] pub use web::*;
#[cfg(target_arch = "wasm32")] pub use wasm::*;
#[cfg(target_arch = "wasm32")] pub use node::*;

/// Pinned future returned by operations that produce no value on success.
pub type EmptyFuture = Pin<Box<dyn Future<Output = anyhow::Result<()>>>>;

/// Pinned future returned by operations that read raw bytes.
pub type BinaryFuture = Pin<Box<dyn Future<Output = anyhow::Result<Vec<u8>>>>>;

/// Pinned future returned by operations that read UTF-8 text.
pub type TextFuture = Pin<Box<dyn Future<Output = anyhow::Result<String>>>>;

/// Pinned future returned by operations that open a [`WriteStream`].
pub type WriteStreamFuture = Pin<Box<dyn Future<Output = anyhow::Result<Box<dyn WriteStream>>>>>;

/// Pinned future returned by operations that open a [`ReadStream`].
pub type ReadStreamFuture = Pin<Box<dyn Future<Output = anyhow::Result<Box<dyn ReadStream>>>>>;

/// A backend for reading and, optionally, writing files by path. Only the
/// read methods are required; write and streaming methods default to
/// returning an error for backends that don't support them (e.g. the
/// HTTP-backed `WebFileSystem` is read-only).
pub trait FileSystem: Send + Sync {
    fn read_bytes(&self, path: String) -> BinaryFuture;
    fn read_text(&self, path: String) -> TextFuture;
    
    #[allow(unused)]
    fn write_bytes(&self, path: String, bytes: Vec<u8>) -> EmptyFuture { 
        Box::pin(async move { bail!("Immutable file system") }) 
    }

    #[allow(unused)]
    fn write_text(&self, path: String, text: String) -> EmptyFuture { 
        Box::pin(async move { bail!("Immutable file system") }) 
    }

    #[allow(unused)]
    fn as_write_stream(&self, path: String) -> WriteStreamFuture {
        Box::pin(async move { bail!("Write stream not supported") }) 
    }
    
    #[allow(unused)]
    fn as_read_stream(&self, path: String) -> ReadStreamFuture { 
        Box::pin(async move { bail!("Read stream not supported") }) 
    }
}

/// A file opened for appending bytes in chunks.
pub trait WriteStream: Send + Sync {
    fn append_bytes(&mut self, bytes: Vec<u8>) -> EmptyFuture;
}

/// A file opened for reading bytes in fixed-size chunks.
pub trait ReadStream: Send + Sync {
    fn stream_bytes(&mut self, count: usize) -> BinaryFuture;
}
