use std::pin::Pin;

use anyhow::bail;

#[cfg(not(target_arch = "wasm32"))] pub mod local;
#[cfg(not(target_arch = "wasm32"))] pub mod web;
#[cfg(target_arch = "wasm32")] pub mod wasm;
#[cfg(all(target_arch = "wasm32", feature = "node"))] pub mod node;

#[cfg(not(target_arch = "wasm32"))] pub use local::*;
use mutual::RefGuard;
#[cfg(not(target_arch = "wasm32"))] pub use web::*;
#[cfg(target_arch = "wasm32")] pub use wasm::*;
#[cfg(all(target_arch = "wasm32", feature = "node"))] pub use node::*;

/// Pinned future returned by operations that produce no value on success.
pub type EmptyFuture = Pin<Box<dyn Future<Output = anyhow::Result<()>>>>;

/// Pinned future returned by operations that read raw bytes.
pub type BinaryFuture = Pin<Box<dyn Future<Output = anyhow::Result<Vec<u8>>>>>;

/// Pinned future returned by operations that read UTF-8 text.
pub type TextFuture = Pin<Box<dyn Future<Output = anyhow::Result<String>>>>;

/// Pinned future returned by operations that answer yes or no.
pub type BoolFuture = Pin<Box<dyn Future<Output = anyhow::Result<bool>>>>;

/// Pinned future returned by operations that list names.
pub type NamesFuture = Pin<Box<dyn Future<Output = anyhow::Result<Vec<String>>>>>;

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

    /// Moves the file or directory at `from` to `to`, replacing a file already there.
    #[allow(unused)]
    fn rename(&self, from: String, to: String) -> EmptyFuture {
        Box::pin(async move { bail!("Rename not supported") })
    }

    /// Makes the directory `path` and any missing parents.
    #[allow(unused)]
    fn create_dir_all(&self, path: String) -> EmptyFuture {
        Box::pin(async move { bail!("Directories not supported") })
    }

    /// Names of the entries in the directory `path`.
    #[allow(unused)]
    fn list_dir(&self, path: String) -> NamesFuture {
        Box::pin(async move { bail!("Directory listing not supported") })
    }

    /// Whether a file or directory is at `path`.
    #[allow(unused)]
    fn exists(&self, path: String) -> BoolFuture {
        Box::pin(async move { bail!("Exists not supported") })
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

/// A wrapper around a ref guard to a file system.  Useful for allowing
/// RefGuard<Box<dyn FileSystem>> to be shared without generics so they
/// may be used by interops. The guard is private because wasm_bindgen
/// exports public fields, which `RefGuard` cannot be.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
pub struct FileSystemRef(RefGuard<Box<dyn FileSystem>>);

impl FileSystemRef {
    pub fn new(guard: RefGuard<Box<dyn FileSystem>>) -> Self {
        Self(guard)
    }
}

impl FileSystem for FileSystemRef {
    fn read_bytes(&self, path: String) -> BinaryFuture {
        self.0.read_bytes(path)
    }

    fn read_text(&self, path: String) -> TextFuture {
        self.0.read_text(path)
    }

    fn write_bytes(&self, path: String, bytes: Vec<u8>) -> EmptyFuture { 
        self.0.write_bytes(path, bytes)
    }

    fn write_text(&self, path: String, text: String) -> EmptyFuture { 
        self.0.write_text(path, text)
    }

    fn as_write_stream(&self, path: String) -> WriteStreamFuture {
        self.0.as_write_stream(path)
    }

    fn as_read_stream(&self, path: String) -> ReadStreamFuture { 
        self.0.as_read_stream(path)
    }

    fn rename(&self, from: String, to: String) -> EmptyFuture {
        self.0.rename(from, to)
    }

    fn create_dir_all(&self, path: String) -> EmptyFuture {
        self.0.create_dir_all(path)
    }

    fn list_dir(&self, path: String) -> NamesFuture {
        self.0.list_dir(path)
    }

    fn exists(&self, path: String) -> BoolFuture {
        self.0.exists(path)
    }
}

/// JS-facing methods. Each returns a `Promise`; streaming is not exposed.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
impl FileSystemRef {
    #[wasm_bindgen(js_name = readBytes)]
    pub fn js_read_bytes(&self, path: String) -> js_sys::Promise {
        promise(FileSystem::read_bytes(self, path), |b| {
            js_sys::Uint8Array::from(b.as_slice()).into()
        })
    }

    #[wasm_bindgen(js_name = readText)]
    pub fn js_read_text(&self, path: String) -> js_sys::Promise {
        promise(FileSystem::read_text(self, path), wasm_bindgen::JsValue::from)
    }

    #[wasm_bindgen(js_name = writeBytes)]
    pub fn js_write_bytes(&self, path: String, bytes: Vec<u8>) -> js_sys::Promise {
        promise(FileSystem::write_bytes(self, path, bytes), |_| wasm_bindgen::JsValue::UNDEFINED)
    }

    #[wasm_bindgen(js_name = writeText)]
    pub fn js_write_text(&self, path: String, text: String) -> js_sys::Promise {
        promise(FileSystem::write_text(self, path, text), |_| wasm_bindgen::JsValue::UNDEFINED)
    }
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn promise<T: 'static>(
    future: Pin<Box<dyn Future<Output = anyhow::Result<T>>>>,
    map: impl Fn(T) -> wasm_bindgen::JsValue + 'static,
) -> js_sys::Promise {
    wasm_bindgen_futures::future_to_promise(async move {
        future
            .await
            .map(map)
            .map_err(|e| wasm_bindgen::JsValue::from(js_sys::Error::new(&e.to_string())))
    })
}
