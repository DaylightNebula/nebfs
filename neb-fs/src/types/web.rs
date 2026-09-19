use std::sync::OnceLock;

use mutual::{RefGuard, RelaxedMutex, SharedData};

use crate::{BinaryFuture, FileSystem};

/// rustls is built without a provider so no C/assembly crypto is linked in;
/// install the pure-Rust one before the first TLS handshake.
fn install_crypto_provider() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let _ = rustls_rustcrypto::provider().install_default();
    });
}

static WEB_FILE_SYSTEM: OnceLock<RelaxedMutex<Box<dyn FileSystem>>> = OnceLock::new();

pub fn web_file_system() -> RefGuard<Box<dyn FileSystem>> {
    let mutex = WEB_FILE_SYSTEM
        .get_or_init(|| RelaxedMutex::new(Box::new(WebFileSystem)));
    return mutex.lock_ref();
}

/// [`FileSystem`] that reads files via plain HTTP GET (through `reqwest`).
/// Read-only: writes fall back to the trait's default "unsupported" error.
pub struct WebFileSystem;
impl FileSystem for WebFileSystem {
    fn read_bytes(&self, path: String) -> BinaryFuture {
        Box::pin(async move {
            install_crypto_provider();
            let response = reqwest::get(path).await?;
            let bytes = response.bytes().await?;
            Ok(Vec::from(bytes))
        })
    }

    fn read_text(&self, path: String) -> super::TextFuture {
        Box::pin(async move {
            install_crypto_provider();
            let response = reqwest::get(path).await?;
            let text = response.text().await?;
            Ok(text)
        })
    }
}
