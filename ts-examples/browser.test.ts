// OPFS backend: needs a real browser, so this reports into the page.
// `npm run build && npm start`, then open the printed URL (see README.md).
import init, { wasm_file_system, VirtualFile } from "../neb-fs/pkg-web/neb_fs.js";
import { tests, type Backend } from "./suite.ts";

const output = document.getElementById("results")!;

function report(name: string, error?: unknown): void {
    const line = document.createElement("li");
    line.textContent = error ? `FAIL  ${name} - ${error}` : `ok    ${name}`;
    console.log(line.textContent);
    line.style.color = error ? "crimson" : "green";
    output.append(line);
}

await init();

// The origin private file system persists across reloads, so start clean.
const dir = "neb-fs-example";
await navigator.storage.getDirectory()
    .then((root) => root.removeEntry(dir, { recursive: true }))
    .catch(() => {});

const backend: Backend = { fileSystem: wasm_file_system, VirtualFile, dir };

let failed = 0;
for (const [name, run] of tests) {
    try {
        await run(backend);
        report(name);
    } catch (error) {
        failed += 1;
        report(name, error);
    }
}

report(`${tests.length - failed}/${tests.length} passed`);
// browser-runner.ts watches the title to know the run is over.
document.title = failed ? `${failed} failed` : "all passed";
