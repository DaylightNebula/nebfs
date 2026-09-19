// Node backend: `node --experimental-strip-types --test node.test.ts`
// (see README.md for the wasm-pack build it needs first).
import { after, test } from "node:test";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";

import { node_file_system, VirtualFile } from "../neb-fs/pkg-node/neb_fs.js";
import { tests, type Backend } from "./suite.ts";

const dir = await mkdtemp(`${tmpdir()}/neb-fs-`);
after(() => rm(dir, { recursive: true, force: true }));

const backend: Backend = { fileSystem: node_file_system, VirtualFile, dir };

for (const [name, run] of tests) {
    test(name, () => run(backend));
}
