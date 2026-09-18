//! Writes a file into the origin private file system and reads it back.
//!
//! Build for the browser, then load the module from a page:
//! `cargo build --example wasm_read_write --target wasm32-unknown-unknown`

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!("this example only runs in a browser; build it with --target wasm32-unknown-unknown");
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use neb_fs::{File, WASM_FILE_SYSTEM};

    const PATH: &'static str = "examples/hello.txt";

    wasm_bindgen_futures::spawn_local(async {
        let file = File::open(WASM_FILE_SYSTEM, PATH);
        file.write_text("hello from nebfs").await.unwrap();

        let text = file.read_text().await.unwrap();
        web_sys::console::log_1(&text.into());

        let bytes = file.read_bytes().await.unwrap();
        assert_eq!(bytes, b"hello from nebfs");
    });
}
