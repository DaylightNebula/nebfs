use anyhow::{Context, anyhow};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    File, FileSystemDirectoryHandle, FileSystemFileHandle, FileSystemGetDirectoryOptions,
    FileSystemGetFileOptions, FileSystemWritableFileStream,
};

use crate::{BinaryFuture, EmptyFuture, FileSystem, TextFuture};

/// Shared instance of [`WasmFileSystem`].
pub const WASM_FILE_SYSTEM: &'static WasmFileSystem = &WasmFileSystem;

/// Browser file system backed by the origin private file system, reading
/// through `web_sys::File` and writing through a writable file stream.
pub struct WasmFileSystem;

fn js_err(value: JsValue) -> anyhow::Error {
    anyhow!("{:?}", value)
}

/// Walks `path` and returns a handle to its final component. Missing
/// directories and files are created when `create` is set.
async fn file_handle(path: &str, create: bool) -> anyhow::Result<FileSystemFileHandle> {
    let mut parts = path.split('/').filter(|part| !part.is_empty()).peekable();
    let storage = web_sys::window()
        .context("no window")?
        .navigator()
        .storage();
    let mut dir: FileSystemDirectoryHandle = JsFuture::from(storage.get_directory())
        .await
        .map_err(js_err)?
        .unchecked_into();

    let mut name = parts.next().context("empty path")?;
    while parts.peek().is_some() {
        let options = FileSystemGetDirectoryOptions::new();
        options.set_create(create);
        dir = JsFuture::from(dir.get_directory_handle_with_options(name, &options))
            .await
            .map_err(js_err)?
            .unchecked_into();
        name = parts.next().unwrap();
    }

    let options = FileSystemGetFileOptions::new();
    options.set_create(create);
    Ok(JsFuture::from(dir.get_file_handle_with_options(name, &options))
        .await
        .map_err(js_err)?
        .unchecked_into())
}

async fn read_file(path: &str) -> anyhow::Result<File> {
    let handle = file_handle(path, false).await?;
    Ok(JsFuture::from(handle.get_file())
        .await
        .map_err(js_err)?
        .unchecked_into())
}

async fn writer(path: &str) -> anyhow::Result<FileSystemWritableFileStream> {
    let handle = file_handle(path, true).await?;
    Ok(JsFuture::from(handle.create_writable())
        .await
        .map_err(js_err)?
        .unchecked_into())
}

async fn close(stream: FileSystemWritableFileStream) -> anyhow::Result<()> {
    JsFuture::from(stream.close()).await.map_err(js_err)?;
    Ok(())
}

impl FileSystem for WasmFileSystem {
    fn read_bytes<'a>(&self, path: &'a str) -> BinaryFuture<'a> {
        Box::pin(async move {
            let buffer = JsFuture::from(read_file(path).await?.array_buffer())
                .await
                .map_err(js_err)?;
            Ok(js_sys::Uint8Array::new(&buffer).to_vec())
        })
    }

    fn read_text<'a>(&self, path: &'a str) -> TextFuture<'a> {
        Box::pin(async move {
            let text = JsFuture::from(read_file(path).await?.text())
                .await
                .map_err(js_err)?;
            text.as_string().context("file text was not a string")
        })
    }

    fn write_bytes<'a>(&self, path: &'a str, mut bytes: Vec<u8>) -> EmptyFuture<'a> {
        Box::pin(async move {
            let stream = writer(path).await?;
            JsFuture::from(
                stream
                    .write_with_u8_array(&mut bytes)
                    .map_err(js_err)?,
            )
            .await
            .map_err(js_err)?;
            close(stream).await
        })
    }

    fn write_text<'a>(&self, path: &'a str, text: String) -> EmptyFuture<'a> {
        Box::pin(async move {
            let stream = writer(path).await?;
            JsFuture::from(stream.write_with_str(&text).map_err(js_err)?)
                .await
                .map_err(js_err)?;
            close(stream).await
        })
    }
}
