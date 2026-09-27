use crate::{FileSystem, FileSystemRef, ReadStream, WriteStream};

/// A path bound to a [`FileSystem`], so reads, writes, and streams can be
/// called without repeating the backend and path at each call site.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
pub struct VirtualFile {
    file_system: FileSystemRef,
    path: String
}

impl VirtualFile {
    /// Binds `path` to `file_system` for subsequent operations.
    pub fn open(file_system: FileSystemRef, path: String) -> VirtualFile {
        Self { file_system, path: path }
    }

    /// Reads the whole file as bytes.
    pub async fn read_bytes(&self) -> anyhow::Result<Vec<u8>> {
        self.file_system.read_bytes(self.path.clone()).await
    }

    /// Reads the whole file as UTF-8 text.
    pub async fn read_text(&self) -> anyhow::Result<String> {
        self.file_system.read_text(self.path.clone()).await
    }

    /// Overwrites the file with `bytes`.
    pub async fn write_bytes(&self, bytes: Vec<u8>) -> anyhow::Result<()> {
        self.file_system.write_bytes(self.path.clone(), bytes).await
    }

    /// Overwrites the file with `text`.
    pub async fn write_text(&self, text: String) -> anyhow::Result<()> {
        self.file_system.write_text(self.path.clone(), text).await
    }

    /// Opens the file for reading in fixed-size chunks.
    pub async fn as_read_stream(&self) -> anyhow::Result<Box<dyn ReadStream>> {
        self.file_system.as_read_stream(self.path.clone()).await
    }

    /// Opens the file for appending in chunks, creating it if needed.
    pub async fn as_write_stream(&self) -> anyhow::Result<Box<dyn WriteStream>> {
        self.file_system.as_write_stream(self.path.clone()).await
    }

    /// Moves the file to `to`, replacing a file already there. The handle keeps its old path.
    pub async fn rename(&self, to: String) -> anyhow::Result<()> {
        self.file_system.rename(self.path.clone(), to).await
    }

    /// Whether the file exists.
    pub async fn exists(&self) -> anyhow::Result<bool> {
        self.file_system.exists(self.path.clone()).await
    }
}

/// JS-facing methods. Each returns a `Promise`; streaming is not exposed.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
impl VirtualFile {
    #[wasm_bindgen::prelude::wasm_bindgen(constructor)]
    pub fn js_open(file_system: FileSystemRef, path: String) -> VirtualFile {
        Self::open(file_system, path)
    }

    #[wasm_bindgen::prelude::wasm_bindgen(js_name = readBytes)]
    pub fn js_read_bytes(&self) -> js_sys::Promise {
        crate::promise(self.file_system.read_bytes(self.path.clone()), |b| {
            js_sys::Uint8Array::from(b.as_slice()).into()
        })
    }

    #[wasm_bindgen::prelude::wasm_bindgen(js_name = readText)]
    pub fn js_read_text(&self) -> js_sys::Promise {
        crate::promise(self.file_system.read_text(self.path.clone()), wasm_bindgen::JsValue::from)
    }

    #[wasm_bindgen::prelude::wasm_bindgen(js_name = writeBytes)]
    pub fn js_write_bytes(&self, bytes: Vec<u8>) -> js_sys::Promise {
        crate::promise(self.file_system.write_bytes(self.path.clone(), bytes), |_| wasm_bindgen::JsValue::UNDEFINED)
    }

    #[wasm_bindgen::prelude::wasm_bindgen(js_name = writeText)]
    pub fn js_write_text(&self, text: String) -> js_sys::Promise {
        crate::promise(self.file_system.write_text(self.path.clone(), text), |_| wasm_bindgen::JsValue::UNDEFINED)
    }
}
