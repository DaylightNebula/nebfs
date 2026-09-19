//! Small async file system abstraction with pluggable backends: local disk
//! (via tokio), the browser origin private file system (wasm), Node.js
//! `fs/promises` (wasm), and plain HTTP GET (web, via reqwest).

pub mod file;
pub mod types;

pub use file::*;
pub use types::*;
