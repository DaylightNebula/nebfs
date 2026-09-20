import io.github.daylightnebula.nebfs.FileSystem;
import io.github.daylightnebula.nebfs.VirtualFile;

import java.util.concurrent.ExecutionException;

/**
 * Smoke test for the published {@code io.github.daylightnebula:neb-fs-jvm}
 * artifact: round-trips a file through the local backend and confirms a
 * missing-file read surfaces as a catchable error with no leak/double-free.
 * {@code readBytes}/{@code readText}/{@code writeBytes}/{@code writeText}
 * all return {@code CompletableFuture}s — the native call still runs
 * synchronously under the hood, but on a background thread, so it never
 * blocks the calling thread.
 *
 * Build and publish the artifact first (from neb-fs-jvm/java/):
 *   ./gradlew publishToMavenLocal
 *
 * Then, from java-examples/, compile and run against the local repo jar
 * (needs a JDK 22+ compiler/runtime — the FFM API is stable from 22 on, no
 * --enable-preview required):
 *   M2_JAR=~/.m2/repository/io/github/daylightnebula/neb-fs-jvm/0.1.0/neb-fs-jvm-0.1.0.jar
 *   javac -cp "$M2_JAR" Main.java
 *   java --enable-native-access=ALL-UNNAMED -ea -cp ".:$M2_JAR" Main
 *
 * (No LD_LIBRARY_PATH / java.library.path needed — the native library ships
 * inside the jar and is extracted and loaded at class-init time.)
 */
public final class Main {
    public static void main(String[] args) throws Throwable {
        try (VirtualFile file = VirtualFile.open(FileSystem.LOCAL, "panama_test.txt")) {
            String text = file.writeText("hello from Java")
                    .thenCompose(v -> file.readText())
                    .get();
            assert text.equals("hello from Java") : text;
            System.out.println("Panama round-trip OK: " + text);
        }

        // FileSystem.LOCAL is reusable across many VirtualFile.open calls.
        try (VirtualFile a = VirtualFile.open(FileSystem.LOCAL, "panama_test.txt");
             VirtualFile b = VirtualFile.open(FileSystem.LOCAL, "panama_test.txt")) {
            assert a.readText().get().equals(b.readText().get());
            System.out.println("FileSystem.LOCAL reuse OK");
        }

        try (VirtualFile missing = VirtualFile.open(FileSystem.LOCAL, "does_not_exist.txt")) {
            missing.readBytes().get();
            throw new AssertionError("expected read of missing file to fail");
        } catch (ExecutionException expected) {
            System.out.println("Missing-file error surfaced correctly: " + expected.getCause().getMessage());
        }
    }
}
