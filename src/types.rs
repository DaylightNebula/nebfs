use std::pin::Pin;

#[cfg(not(target_arch = "wasm32"))] pub mod local;
#[cfg(target_arch = "wasm32")] pub mod wasm;
#[cfg(target_arch = "wasm32")] pub mod node;
#[cfg(not(target_os = "jvm"))] pub mod web;

#[cfg(not(target_arch = "wasm32"))] pub use local::*;
#[cfg(target_arch = "wasm32")] pub use wasm::*;
#[cfg(target_arch = "wasm32")] pub use node::*;
#[cfg(not(target_os = "jvm"))] pub use web::*;

pub type EmptyFuture<'a> = Pin<Box<dyn Future<Output = anyhow::Result<()>> + 'a>>;
pub type BinaryFuture<'a> = Pin<Box<dyn Future<Output = anyhow::Result<Vec<u8>>> + 'a>>;
pub type TextFuture<'a> = Pin<Box<dyn Future<Output = anyhow::Result<String>> + 'a>>;

pub trait FileSystem: Send + Sync {
    fn read_bytes<'a>(&self, path: &'a str) -> BinaryFuture<'a>;
    fn read_text<'a>(&self, path: &'a str) -> TextFuture<'a>;
    fn write_bytes<'a>(&self, path: &'a str, bytes: Vec<u8>) -> EmptyFuture<'a>;
    fn write_text<'a>(&self, path: &'a str, text: String) -> EmptyFuture<'a>;
}