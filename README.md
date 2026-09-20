# neb-fs

Status: work in progress (0.1.0), API may still change.

A small async file system abstraction with pluggable backends allowing local files, HTTP files, JS files
and others to be treated the same.  Currently this project provides a local backend and a web backend
for native targets, a backend for handling files on node, and a backend for interacting with JS files. 

## Project layout

This is a multi-crate repo:

- [`neb-fs/`](neb-fs) — the core Rust library described below.
- [`neb-fs-js/`](neb-fs-js) — JS bindings, in progress (crate scaffolded, no bindings yet).
- [`neb-fs-jvm/`](neb-fs-jvm) — a native cdylib exposing the local and web
  backends over a plain C ABI, for calling from Java (or any JVM language)
  via Project Panama. No JNI, no bytecode-compilation backend involved.
  [`neb-fs-jvm/java/`](neb-fs-jvm/java) holds the Java bindings, packaged as
  a self-contained jar publishable to Maven local.
- [`ts-examples/`](ts-examples) — runnable TypeScript examples for the Node and browser backends.
- [`java-examples/`](java-examples) — a runnable Java example for the `neb-fs-jvm` bindings.

## Backends

| Backend | Type | Target | Read | Write | Streaming |
|---|---|---|---|---|---|
| Local disk (tokio) | `LocalFileSystem` / `LOCAL_FILE_SYSTEM` | non-wasm | yes | yes | yes |
| Browser origin private file system | `WasmFileSystem` / `WASM_FILE_SYSTEM` | wasm32 | yes | yes | no |
| Node.js `fs/promises` | `NodeFileSystem` / `NODE_FILE_SYSTEM` | wasm32 | yes | yes | no |
| HTTP GET (reqwest) | `WebFileSystem` / `WEB_FILE_SYSTEM` | any | yes | no | no |
| JVM, via Project Panama | `neb-fs-jvm` cdylib | any JVM (Java/Kotlin/...) | yes | local only | no |

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

### JVM (Java, via Project Panama)

`neb-fs-jvm` builds `neb-fs`'s local and web backends as an ordinary native
shared library (a `cdylib`), called from Java through the
`java.lang.foreign` Foreign Function & Memory API — no JNI, no generated
stubs, no bytecode-compilation backend involved. `FileSystem.LOCAL` /
`FileSystem.WEB` mirror `local_file_system()` / `web_file_system()` as
reusable constants, and `VirtualFile.open(fileSystem, path)` mirrors the
Rust/JS API shape. `open` and `close` are plain synchronous calls (neither
touches the disk — `open` just pairs a path with a backend, `close` frees
the native handle, and `close` has to be synchronous anyway to satisfy
`AutoCloseable`), but `readBytes`/`writeBytes`/`readText`/`writeText` — the
ones that actually do file I/O — each return a `CompletableFuture`: the
native call still runs synchronously under the hood (there's no async
runtime on the other side of the FFI boundary to hand a callback to), but
it runs on a background thread, so the calling thread never blocks on it:

```java
try (VirtualFile file = VirtualFile.open(FileSystem.LOCAL, "notes/todo.txt")) {
    file.writeText("write me")
        .thenCompose(v -> file.readText())
        .thenAccept(System.out::println)
        .join();
}
```

Don't `close()` a `VirtualFile` while one of its futures is still pending —
same caveat as calling any of its methods concurrently from another thread.

Requires a **JDK 22+** compiler and runtime — the FFM API was finalized in
JDK 22 (JEP 454), so no `--enable-preview` is needed. (It's a preview
feature on JDK 21, and preview class files only run on the exact JDK build
that produced them — a jar built with `--release 21 --enable-preview` will
fail with `UnsupportedClassVersionError` on any other JDK, including newer
ones, which is why `neb-fs-jvm` targets `--release 22` instead: consumers on
JDK 22 through the latest LTS all load the same jar.) Running still wants
`--enable-native-access=ALL-UNNAMED` to silence the native-access warning.

#### Building the jar and publishing to Maven local

[`neb-fs-jvm/java/`](neb-fs-jvm/java) holds the Java sources
(`io.github.daylightnebula.nebfs`). `build.sh` builds the native library,
compiles the bindings, and packs both into a single jar with the native
library under `native/` — self-contained, no `LD_LIBRARY_PATH` or
`java.library.path` setup needed by consumers, since `VirtualFile` extracts
and loads it from the jar's own classpath resources on first use:

```bash
cd neb-fs-jvm/java
./build.sh
mvn install:install-file -Dfile=build/neb-fs-jvm-0.1.0.jar \
  -DgroupId=io.github.daylightnebula -DartifactId=neb-fs-jvm \
  -Dversion=0.1.0 -Dpackaging=jar
```

That installs it to `~/.m2/repository`, so any local Maven (or Gradle
`mavenLocal()`) project can depend on `io.github.daylightnebula:neb-fs-jvm:0.1.0`.
[`java-examples/Main.java`](java-examples/Main.java) is a working smoke test —
read/write round trip, `FileSystem` reuse across multiple files, and the
missing-file error path — built and run against the published jar; see the
commands in its header comment.

Streaming isn't exposed over this boundary (same as the wasm/node
backends) — it would need its own opaque handle lifetime story on top of
what whole-file read/write needs. Nor is cross-platform packaging: the jar
bundles a single native library built for the host it was built on (see the
`ponytail:` note in `VirtualFile.loadBundledLibrary`) — publishing for other
OS/architectures needs per-platform native resources and a lookup by
`os.name`/`os.arch`.

## Implementing a custom backend

Implement the `FileSystem` trait (see [`neb-fs/src/types.rs`](neb-fs/src/types.rs))
— only `read_bytes` and `read_text` are required, write and streaming
methods are optional.
