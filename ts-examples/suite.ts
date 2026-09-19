/** Backend-agnostic checks, run against both the Node and OPFS file systems. */

/** The subset of `FileSystemRef` both backends expose to JS. */
export interface Fs {
    readBytes(path: string): Promise<any>;
    readText(path: string): Promise<any>;
    writeBytes(path: string, bytes: Uint8Array): Promise<any>;
    writeText(path: string, text: string): Promise<any>;
}

/** The subset of `VirtualFile` both backends expose to JS. */
export interface File {
    readBytes(): Promise<any>;
    readText(): Promise<any>;
    writeBytes(bytes: Uint8Array): Promise<any>;
    writeText(text: string): Promise<any>;
}

export interface Backend {
    /** Opens a fresh handle. Handles are cheap, and `new VirtualFile(fs, ...)`
     *  consumes the one it is given, so never reuse one across calls. */
    fileSystem(): Fs;
    /** The generated `VirtualFile` class of whichever package is loaded.
     *  `fs` is `any` because each package generates its own nominal type. */
    VirtualFile: new (fs: any, path: string) => File;
    /** Directory the checks may write into, without a trailing slash. */
    dir: string;
}

function assert(ok: boolean, message: string): void {
    if (!ok) throw new Error(message);
}

function eq(actual: unknown, expected: unknown, message: string): void {
    assert(actual === expected, `${message}: expected ${expected}, got ${actual}`);
}

export const tests: [string, (backend: Backend) => Promise<void>][] = [
    ["text round-trip", async ({ fileSystem, dir }) => {
        const fs = fileSystem();
        await fs.writeText(`${dir}/hello.txt`, "hello");
        eq(await fs.readText(`${dir}/hello.txt`), "hello", "text");
    }],

    ["byte round-trip", async ({ fileSystem, dir }) => {
        const fs = fileSystem();
        await fs.writeBytes(`${dir}/bytes.bin`, new Uint8Array([1, 2, 255]));
        const bytes: Uint8Array = await fs.readBytes(`${dir}/bytes.bin`);
        eq(bytes.join(","), "1,2,255", "bytes");
    }],

    ["overwrites, never appends", async ({ fileSystem, dir }) => {
        const fs = fileSystem();
        await fs.writeText(`${dir}/over.txt`, "first pass");
        await fs.writeText(`${dir}/over.txt`, "second");
        eq(await fs.readText(`${dir}/over.txt`), "second", "overwritten text");
    }],

    ["missing parent directories are created", async ({ fileSystem, dir }) => {
        const fs = fileSystem();
        await fs.writeText(`${dir}/a/b/c/deep.txt`, "deep");
        eq(await fs.readText(`${dir}/a/b/c/deep.txt`), "deep", "nested text");
    }],

    ["VirtualFile binds a path", async ({ fileSystem, dir, VirtualFile }) => {
        const file = new VirtualFile(fileSystem(), `${dir}/bound.txt`);
        await file.writeText("bound");
        eq(await file.readText(), "bound", "bound text");

        await file.writeBytes(new Uint8Array([7, 8]));
        const bytes: Uint8Array = await file.readBytes();
        eq(bytes.join(","), "7,8", "bound bytes");
    }],

    ["reading a missing file rejects", async ({ fileSystem, dir }) => {
        const fs = fileSystem();
        await fs.readText(`${dir}/nope.txt`).then(
            () => assert(false, "expected a rejection"),
            () => {},
        );
    }],
];
