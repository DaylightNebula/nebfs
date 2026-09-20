package io.github.daylightnebula.nebfs;

/** A backend selector passed into {@link VirtualFile#open}. */
public final class FileSystem {
    static final int LOCAL_KIND = 0;
    static final int WEB_KIND = 1;

    /** Local disk, backed by {@code neb_fs::local_file_system()}. Read/write. */
    public static final FileSystem LOCAL = new FileSystem(LOCAL_KIND);

    /** Plain HTTP GET, backed by {@code neb_fs::web_file_system()}. Read-only. */
    public static final FileSystem WEB = new FileSystem(WEB_KIND);

    final int kind;

    private FileSystem(int kind) {
        this.kind = kind;
    }
}
