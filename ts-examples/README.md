# neb-fs JS/TS examples

Two runnable examples, both driving the same checks in [`suite.ts`](suite.ts):

| Backend | Entry point | What it uses |
| --- | --- | --- |
| Node | [`node.test.ts`](node.test.ts) | `node_file_system()` on `node:fs/promises` |
| Browser | [`browser.test.ts`](browser.test.ts) | `wasm_file_system()` on the origin private file system |

## Running

```sh
npm install
npm test            # both backends
npm run test:node   # Node only
npm run test:browser  # headless Chromium only
```

Each script builds the wasm package it needs first. The browser run serves the
repo over `http://localhost:8123` (the origin private file system needs a
secure context) and drives it with `playwright-core`. It picks the first
Chromium under `~/.cache/ms-playwright`; set `$CHROME` to use another one.

To watch it in a real browser instead:

```sh
npm start           # then open http://localhost:8000/jsts-examples/index.html
```

## Two packages, one crate

The Node backend imports `node:fs/promises`, which a browser cannot resolve, so
it lives behind the crate's `node` feature:

```sh
wasm-pack build ../neb-fs --target nodejs --out-dir pkg-node -- --features node
wasm-pack build ../neb-fs --target web    --out-dir pkg-web
```

## Handles are moved, not shared

`new VirtualFile(fs, path)` takes ownership of the `FileSystemRef` it is given,
and wasm-bindgen frees the JS wrapper at that point. Open a fresh handle per
use - they are cheap, and all of them point at the same backend:

```ts
const file = new VirtualFile(node_file_system(), "notes/todo.txt");
await file.writeText("write me");
```
