use std::pin::Pin;

#[cfg(not(target_arch = "wasm32"))] pub mod local;
#[cfg(target_arch = "wasm32")] pub mod wasm;
#[cfg(target_arch = "wasm32")] pub mod node;
pub mod web;

use anyhow::bail;
#[cfg(not(target_arch = "wasm32"))] pub use local::*;
#[cfg(target_arch = "wasm32")] pub use wasm::*;
#[cfg(target_arch = "wasm32")] pub use node::*;
pub use web::*;

pub type EmptyFuture<'a> = Pin<Box<dyn Future<Output = anyhow::Result<()>> + 'a>>;
pub type BinaryFuture<'a> = Pin<Box<dyn Future<Output = anyhow::Result<Vec<u8>>> + 'a>>;
pub type TextFuture<'a> = Pin<Box<dyn Future<Output = anyhow::Result<String>> + 'a>>;
pub type WriteStreamFuture<'a> = Pin<Box<dyn Future<Output = anyhow::Result<Box<dyn WriteStream>>> + 'a>>;
pub type ReadStreamFuture<'a> = Pin<Box<dyn Future<Output = anyhow::Result<Box<dyn ReadStream>>> + 'a>>;

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

pub trait WriteStream: Send + Sync {
    fn append_bytes<'a>(&'a mut self, bytes: Vec<u8>) -> EmptyFuture<'a>;
}

pub trait ReadStream: Send + Sync {
    fn stream_bytes<'a>(&'a mut self, count: usize) -> BinaryFuture<'a>;
}
