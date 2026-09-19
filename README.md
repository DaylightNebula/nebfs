# neb-fs

Status: work in progress (0.1.0), API may still change.

A small async file system abstraction with pluggable backends allowing local files, HTTP files, JS files
and others to be treated the same.  Currently this project provides a local backend and a web backend
for native targets, a backend for handling files on node, and a backend for interacting with JS files. 

## Project layout

This is a multi-crate repo:

- [`neb-fs/`](neb-fs) — the core Rust library described below.
- [`neb-fs-js/`](neb-fs-js) — JS bindings, in progress (crate scaffolded, no bindings yet).
- `neb-fs-jvm` — JVM bindings, planned, not started yet.
- [`jsts-examples/`](jsts-examples) — runnable TypeScript examples for the Node and browser backends.

## Backends

| Backend | Type | Target | Read | Write | Streaming |
|---|---|---|---|---|---|
| Local disk (tokio) | `LocalFileSystem` / `LOCAL_FILE_SYSTEM` | non-wasm | yes | yes | yes |
| Browser origin private file system | `WasmFileSystem` / `WASM_FILE_SYSTEM` | wasm32 | yes | yes | no |
| Node.js `fs/promises` | `NodeFileSystem` / `NODE_FILE_SYSTEM` | wasm32 | yes | yes | no |
| HTTP GET (reqwest) | `WebFileSystem` / `WEB_FILE_SYSTEM` | any | yes | no | no |

Backends that don't support write or streaming fall back to the
`FileSystem` trait's default methods, which return an error.

## Usage

Open a path against a backend with `File::open`, then read or write it:

```rust
use neb_fs::{File, LOCAL_FILE_SYSTEM};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let file = File::open(LOCAL_FILE_SYSTEM, "test.txt");
    file.write_text("HELLO WORLD\nTHIS IS SOME test text!").await?;
    let text = file.read_text().await?;
    println!("{text}");
    Ok(())
}
```

Fetch a remote file over HTTP the same way, using `WEB_FILE_SYSTEM`:

```rust
use neb_fs::{File, WEB_FILE_SYSTEM};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let file = File::open(WEB_FILE_SYSTEM, "https://example.com/image.png");
    let bytes = file.read_bytes().await?;
    std::fs::write("output.png", bytes)?;
    Ok(())
}
```

### Streaming

For large files, stream reads and writes in chunks instead of loading the
whole file into memory:

```rust
use neb_fs::{File, LOCAL_FILE_SYSTEM};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let file = File::open(LOCAL_FILE_SYSTEM, "big.txt");
    let mut stream = file.as_write_stream().await?;
    stream.append_bytes("hello".into()).await?;
    stream.append_bytes("world".into()).await?;
    Ok(())
}
```

```rust
use neb_fs::{File, LOCAL_FILE_SYSTEM};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let file = File::open(LOCAL_FILE_SYSTEM, "big.txt");
    let mut stream = file.as_read_stream().await?;
    while let Ok(bytes) = stream.stream_bytes(1024).await {
        if bytes.is_empty() { break }
        // process bytes
    }
    Ok(())
}
```

### Browser (wasm)

`WasmFileSystem` reads and writes through the browser's origin private file
system. Build the example for `wasm32-unknown-unknown` and load it from a
page:

```bash
cargo build --manifest-path neb-fs/Cargo.toml --example wasm_read_write --target wasm32-unknown-unknown
```

See [`neb-fs/examples/wasm_read_write.rs`](neb-fs/examples/wasm_read_write.rs).

More examples live in [`neb-fs/examples/`](neb-fs/examples): reading
(`local_read`, `local_read_stream`), writing (`local_write`,
`local_write_stream`), and fetching over HTTP (`web_read`).

### JavaScript / TypeScript

`wasm-pack` exports `FileSystemRef` and `VirtualFile` to JS, with `readBytes`,
`readText`, `writeBytes` and `writeText` returning promises. Streaming is not
exposed.

The Node backend imports `node:fs/promises`, which a browser cannot resolve, so
it sits behind the `node` feature and each target gets its own package:

```bash
wasm-pack build neb-fs --target nodejs --out-dir pkg-node -- --features node
wasm-pack build neb-fs --target web    --out-dir pkg-web
```

```ts
import { node_file_system, VirtualFile } from "./neb-fs/pkg-node/neb_fs.js";

const file = new VirtualFile(node_file_system(), "notes/todo.txt");
await file.writeText("write me");
console.log(await file.readText());
```

Note that `new VirtualFile(fs, path)` takes ownership of the handle it is
given, so open a fresh one per use. Working examples for both backends, with
checks that run under `npm test`, are in [`jsts-examples/`](jsts-examples).

## Implementing a custom backend

Implement the `FileSystem` trait (see [`neb-fs/src/types.rs`](neb-fs/src/types.rs))
— only `read_bytes` and `read_text` are required, write and streaming
methods are optional.
