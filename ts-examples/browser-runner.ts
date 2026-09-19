// Runs browser.test.ts in headless Chromium and exits non-zero on failure.
// OPFS needs a secure context, so the files are served over localhost.
import { spawn } from "node:child_process";
import { readdirSync } from "node:fs";
import { chromium } from "playwright-core";

/** Playwright pins a browser revision this repo does not track, so take the
 *  first Chromium lying around unless $CHROME points at one. */
function chromiumPath(): string {
    if (process.env.CHROME) return process.env.CHROME;

    const cache = `${process.env.HOME}/.cache/ms-playwright`;
    const dir = readdirSync(cache).find((entry) => entry.startsWith("chromium-"));
    if (!dir) throw new Error("no Chromium found - set $CHROME to one");
    return `${cache}/${dir}/chrome-linux64/chrome`;
}

const port = 8123;
const server = spawn("python3", ["-m", "http.server", String(port), "--directory", ".."], {
    stdio: "ignore",
});
const browser = await chromium.launch({ executablePath: chromiumPath() });

try {
    const page = await browser.newPage();
    page.on("console", (message) => console.log(message.text()));
    page.on("pageerror", (error) => console.error(error));

    await page.goto(`http://localhost:${port}/jsts-examples/index.html`);
    await page.waitForFunction(() => document.title !== "running", null, { timeout: 30_000 });

    if (await page.title() !== "all passed") process.exitCode = 1;
} finally {
    await browser.close();
    server.kill();
}
