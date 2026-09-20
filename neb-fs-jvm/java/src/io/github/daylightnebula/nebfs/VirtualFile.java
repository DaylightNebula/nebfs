package io.github.daylightnebula.nebfs;

import java.io.IOException;
import java.io.InputStream;
import java.io.UncheckedIOException;
import java.lang.foreign.Arena;
import java.lang.foreign.FunctionDescriptor;
import java.lang.foreign.Linker;
import java.lang.foreign.MemoryLayout;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.SymbolLookup;
import java.lang.foreign.ValueLayout;
import java.lang.invoke.MethodHandle;
import java.lang.invoke.VarHandle;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

/**
 * A path bound to a {@link FileSystem}, calling into {@code neb-fs-jvm}'s
 * native cdylib through Project Panama ({@code java.lang.foreign}). No JNI,
 * no generated stubs.
 */
public final class VirtualFile implements AutoCloseable {
    private static final Linker LINKER = Linker.nativeLinker();
    private static final SymbolLookup LIB = loadBundledLibrary();

    /**
     * Extracts the native library bundled under {@code /native/} in this jar
     * to a temp file and loads it, then returns a lookup over libraries
     * loaded by this class loader.
     *
     * <p>ponytail: bundles a single native library built for the current
     * platform (Linux x86_64 here) rather than picking one of several by OS
     * and architecture. Add {@code /native/<os>-<arch>/...} subfolders and
     * pick among them here if this needs to run on other platforms.
     */
    private static SymbolLookup loadBundledLibrary() {
        String resourceName = "/native/" + System.mapLibraryName("neb_fs_jvm");
        try (InputStream in = VirtualFile.class.getResourceAsStream(resourceName)) {
            if (in == null) {
                throw new IllegalStateException("native library not found in jar: " + resourceName);
            }
            String suffix = resourceName.substring(resourceName.lastIndexOf('.'));
            Path tmp = Files.createTempFile("neb_fs_jvm", suffix);
            tmp.toFile().deleteOnExit();
            Files.copy(in, tmp, StandardCopyOption.REPLACE_EXISTING);
            System.load(tmp.toAbsolutePath().toString());
        } catch (IOException e) {
            throw new UncheckedIOException("failed to extract neb-fs-jvm native library", e);
        }
        return SymbolLookup.loaderLookup();
    }

    private static final MemoryLayout FFI_BUF = MemoryLayout.structLayout(
            ValueLayout.ADDRESS.withName("ptr"),
            ValueLayout.JAVA_LONG.withName("len"));
    private static final VarHandle BUF_PTR = FFI_BUF.varHandle(MemoryLayout.PathElement.groupElement("ptr"));
    private static final VarHandle BUF_LEN = FFI_BUF.varHandle(MemoryLayout.PathElement.groupElement("len"));

    private static MethodHandle handle(String name, FunctionDescriptor fd) {
        return LINKER.downcallHandle(LIB.find(name).orElseThrow(), fd);
    }

    private static final MethodHandle OPEN = handle("neb_fs_open",
            FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.JAVA_BYTE, ValueLayout.ADDRESS, ValueLayout.ADDRESS));
    private static final MethodHandle CLOSE = handle("neb_fs_close",
            FunctionDescriptor.ofVoid(ValueLayout.ADDRESS));
    private static final MethodHandle READ_BYTES = handle("neb_fs_read_bytes",
            FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.ADDRESS, ValueLayout.ADDRESS));
    private static final MethodHandle WRITE_BYTES = handle("neb_fs_write_bytes",
            FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.ADDRESS, ValueLayout.ADDRESS, ValueLayout.JAVA_LONG));
    private static final MethodHandle FREE_BUF = handle("neb_fs_free_buf",
            FunctionDescriptor.ofVoid(FFI_BUF));
    private static final MethodHandle FREE_ERROR = handle("neb_fs_free_error",
            FunctionDescriptor.ofVoid(ValueLayout.ADDRESS));

    // ponytail: unbounded cached pool; each downcall blocks its thread for
    // the duration of the native call (there's no native callback to avoid
    // that), so this just relocates the block off the caller's thread onto
    // one of these. Switch to a bounded executor if callers fire many more
    // concurrent async ops than there are files actually in flight.
    private static final ExecutorService IO_EXECUTOR = Executors.newCachedThreadPool(runnable -> {
        Thread thread = new Thread(runnable, "neb-fs-jvm-io");
        thread.setDaemon(true);
        return thread;
    });

    @FunctionalInterface
    private interface ThrowingSupplier<T> {
        T get() throws Throwable;
    }

    private static <T> CompletableFuture<T> supplyBlocking(ThrowingSupplier<T> supplier) {
        return CompletableFuture.supplyAsync(() -> {
            try {
                return supplier.get();
            } catch (RuntimeException | Error e) {
                throw e;
            } catch (Throwable t) {
                throw new RuntimeException(t);
            }
        }, IO_EXECUTOR);
    }

    private final MemorySegment handle;

    private VirtualFile(MemorySegment handle) {
        this.handle = handle;
    }

    /** Opens {@code path} against {@code fileSystem}. {@code fileSystem} may be reused for many files. */
    public static VirtualFile open(FileSystem fileSystem, String path) throws Throwable {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment cPath = arena.allocateFrom(path);
            MemorySegment errOut = arena.allocate(ValueLayout.ADDRESS);
            MemorySegment result = (MemorySegment) OPEN.invoke((byte) fileSystem.kind, cPath, errOut);
            if (result.equals(MemorySegment.NULL)) {
                throw errorFrom(errOut.get(ValueLayout.ADDRESS, 0));
            }
            return new VirtualFile(result);
        }
    }

    /**
     * Reads the whole file as bytes. Runs on a background thread so the
     * caller isn't blocked; don't {@link #close()} this file while the
     * returned future is still pending.
     */
    public CompletableFuture<byte[]> readBytes() {
        return supplyBlocking(this::readBytesBlocking);
    }

    /** Reads the whole file as UTF-8 text. See {@link #readBytes()}. */
    public CompletableFuture<String> readText() {
        return supplyBlocking(() -> new String(readBytesBlocking(), StandardCharsets.UTF_8));
    }

    /** Overwrites the file with {@code data}. See {@link #readBytes()}. */
    public CompletableFuture<Void> writeBytes(byte[] data) {
        return supplyBlocking(() -> {
            writeBytesBlocking(data);
            return null;
        });
    }

    /** Overwrites the file with {@code text}, encoded as UTF-8. See {@link #readBytes()}. */
    public CompletableFuture<Void> writeText(String text) {
        return writeBytes(text.getBytes(StandardCharsets.UTF_8));
    }

    private byte[] readBytesBlocking() throws Throwable {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment out = arena.allocate(FFI_BUF);
            checkError((MemorySegment) READ_BYTES.invoke(handle, out));
            long len = (long) BUF_LEN.get(out, 0L);
            MemorySegment ptr = ((MemorySegment) BUF_PTR.get(out, 0L)).reinterpret(len);
            byte[] result = ptr.toArray(ValueLayout.JAVA_BYTE);
            FREE_BUF.invoke(out);
            return result;
        }
    }

    private void writeBytesBlocking(byte[] data) throws Throwable {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment cData = arena.allocateFrom(ValueLayout.JAVA_BYTE, data);
            checkError((MemorySegment) WRITE_BYTES.invoke(handle, cData, (long) data.length));
        }
    }

    private static void checkError(MemorySegment err) throws Throwable {
        if (!err.equals(MemorySegment.NULL)) {
            throw errorFrom(err);
        }
    }

    private static RuntimeException errorFrom(MemorySegment err) throws Throwable {
        String message = err.reinterpret(Long.MAX_VALUE).getString(0);
        FREE_ERROR.invoke(err);
        return new RuntimeException(message);
    }

    @Override
    public void close() {
        try {
            CLOSE.invoke(handle);
        } catch (Throwable e) {
            throw new RuntimeException(e);
        }
    }
}
