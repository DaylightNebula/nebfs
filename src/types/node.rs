use anyhow::{Context, anyhow};
use wasm_bindgen::prelude::*;

use crate::{BinaryFuture, EmptyFuture, FileSystem, TextFuture};

pub const NODE_FILE_SYSTEM: &'static NodeFileSystem = &NodeFileSystem;

/// Node file system backed by `node:fs/promises`.
pub struct NodeFileSystem;

#[wasm_bindgen(module = "node:fs/promises")]
extern "C" {
    #[wasm_bindgen(js_name = readFile, catch)]
    async fn read_file(path: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = readFile, catch)]
    async fn read_file_with_encoding(path: &str, encoding: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = writeFile, catch)]
    async fn write_file(path: &str, data: &JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = mkdir, catch)]
    async fn mkdir(path: &str, options: &JsValue) -> Result<JsValue, JsValue>;
}

fn js_err(value: JsValue) -> anyhow::Error {
    anyhow!("{:?}", value)
}

/// Creates the parent directories of `path`, so writes to a nested path succeed
/// the way they do on the OPFS backend.
async fn create_parents(path: &str) -> anyhow::Result<()> {
    let Some(parent) = path.rsplit_once('/').map(|(parent, _)| parent) else {
        return Ok(());
    };
    if parent.is_empty() {
        return Ok(());
    }

    let options = js_sys::Object::new();
    js_sys::Reflect::set(&options, &"recursive".into(), &JsValue::TRUE).map_err(js_err)?;
    mkdir(parent, &options).await.map_err(js_err)?;
    Ok(())
}

impl FileSystem for NodeFileSystem {
    fn read_bytes<'a>(&self, path: &'a str) -> BinaryFuture<'a> {
        Box::pin(async move {
            let buffer = read_file(path).await.map_err(js_err)?;
            Ok(js_sys::Uint8Array::new(&buffer).to_vec())
        })
    }

    fn read_text<'a>(&self, path: &'a str) -> TextFuture<'a> {
        Box::pin(async move {
            let text = read_file_with_encoding(path, "utf8").await.map_err(js_err)?;
            text.as_string().context("file text was not a string")
        })
    }

    fn write_bytes<'a>(&self, path: &'a str, bytes: Vec<u8>) -> EmptyFuture<'a> {
        Box::pin(async move {
            create_parents(path).await?;
            let data = js_sys::Uint8Array::from(bytes.as_slice());
            write_file(path, &data).await.map_err(js_err)?;
            Ok(())
        })
    }

    fn write_text<'a>(&self, path: &'a str, text: String) -> EmptyFuture<'a> {
        Box::pin(async move {
            create_parents(path).await?;
            write_file(path, &JsValue::from_str(&text))
                .await
                .map_err(js_err)?;
            Ok(())
        })
    }
}
