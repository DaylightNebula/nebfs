use anyhow::bail;

use crate::{BinaryFuture, FileSystem};

/// rustls is built without a provider so no C/assembly crypto is linked in;
/// install the pure-Rust one before the first TLS handshake.
fn install_crypto_provider() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let _ = rustls_rustcrypto::provider().install_default();
    });
}

pub const WEB_FILE_SYSTEM: &'static WebFileSystem = &WebFileSystem;

pub struct WebFileSystem;
impl FileSystem for WebFileSystem {
    fn read_bytes<'a>(&self, path: &'a str) -> BinaryFuture<'a> {
        Box::pin(async move {
            install_crypto_provider();
            let response = reqwest::get(path).await?;
            let bytes = response.bytes().await?;
            Ok(Vec::from(bytes))
        })
    }

    fn read_text<'a>(&self, path: &'a str) -> super::TextFuture<'a> {
        Box::pin(async move {
            install_crypto_provider();
            let response = reqwest::get(path).await?;
            let text = response.text().await?;
            Ok(text)
        })
    }

    fn write_bytes<'a>(&self, _path: &'a str, _bytes: Vec<u8>) -> super::EmptyFuture<'a> {
        Box::pin(async move {
            bail!("File system has no write capability")
        })
    }

    fn write_text<'a>(&self, _path: &'a str, _text: String) -> super::EmptyFuture<'a> {
        Box::pin(async move {
            bail!("File system has no write capability")
        })
    }
}
