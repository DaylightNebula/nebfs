//! C ABI shim exposing `neb-fs`'s local/web backends as a native cdylib, for
//! calling from Java (or any JVM language) via Project Panama's
//! `java.lang.foreign` Foreign Function & Memory API. No JNI, no generated
//! stubs.

use std::ffi::{CStr, CString, c_char};
use std::sync::OnceLock;

use anyhow::bail;
use neb_fs::{FileSystemRef, VirtualFile, local_file_system, web_file_system};

/// An owned byte buffer handed across the FFI boundary. Free with
/// [`neb_fs_free_buf`].
#[repr(C)]
pub struct FfiBuf {
    pub ptr: *mut u8,
    pub len: usize,
}

fn runtime() -> &'static tokio::runtime::Runtime {
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to start neb-fs-jvm runtime")
    })
}

fn file_system(kind: u8) -> anyhow::Result<FileSystemRef> {
    match kind {
        0 => Ok(local_file_system()),
        1 => Ok(web_file_system()),
        _ => bail!("unknown file system kind {kind}"),
    }
}

fn ok_buf(bytes: Vec<u8>) -> FfiBuf {
    let mut bytes = bytes.into_boxed_slice();
    let buf = FfiBuf {
        ptr: bytes.as_mut_ptr(),
        len: bytes.len(),
    };
    std::mem::forget(bytes);
    buf
}

fn err_string(message: String) -> *mut c_char {
    CString::new(message).unwrap_or_default().into_raw()
}

/// Opens a [`VirtualFile`] against the backend selected by `kind` (`0` =
/// local, `1` = web) and `path`.
///
/// # Safety
/// `path` must be a valid, NUL-terminated UTF-8 C string. On failure, writes
/// an owned error message (free with [`neb_fs_free_error`]) to `*err_out`
/// and returns null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neb_fs_open(
    kind: u8,
    path: *const c_char,
    err_out: *mut *mut c_char,
) -> *mut VirtualFile {
    let path = unsafe { CStr::from_ptr(path) }.to_string_lossy().into_owned();
    match file_system(kind) {
        Ok(fs) => Box::into_raw(Box::new(VirtualFile::open(fs, path))),
        Err(e) => {
            unsafe { *err_out = err_string(e.to_string()) };
            std::ptr::null_mut()
        }
    }
}

/// Releases a [`VirtualFile`] opened with [`neb_fs_open`].
///
/// # Safety
/// `handle` must be a live pointer returned by [`neb_fs_open`], not
/// previously freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neb_fs_close(handle: *mut VirtualFile) {
    if !handle.is_null() {
        drop(unsafe { Box::from_raw(handle) });
    }
}

/// Reads the whole file into `*out`. Returns null on success, or an owned
/// error message (free with [`neb_fs_free_error`]) on failure.
///
/// # Safety
/// `handle` must be a live pointer from [`neb_fs_open`]; `out` must be valid
/// for writes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neb_fs_read_bytes(handle: *mut VirtualFile, out: *mut FfiBuf) -> *mut c_char {
    match runtime().block_on(unsafe { &*handle }.read_bytes()) {
        Ok(bytes) => {
            unsafe { *out = ok_buf(bytes) };
            std::ptr::null_mut()
        }
        Err(e) => err_string(e.to_string()),
    }
}

/// Overwrites the file with `len` bytes from `data`. Returns null on
/// success, or an owned error message (free with [`neb_fs_free_error`]) on
/// failure.
///
/// # Safety
/// `handle` must be a live pointer from [`neb_fs_open`]; `data` must be
/// valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neb_fs_write_bytes(
    handle: *mut VirtualFile,
    data: *const u8,
    len: usize,
) -> *mut c_char {
    let bytes = unsafe { std::slice::from_raw_parts(data, len) }.to_vec();
    match runtime().block_on(unsafe { &*handle }.write_bytes(bytes)) {
        Ok(()) => std::ptr::null_mut(),
        Err(e) => err_string(e.to_string()),
    }
}

/// Frees a buffer produced by [`neb_fs_read_bytes`].
///
/// # Safety
/// `buf` must be a buffer produced by [`neb_fs_read_bytes`], not previously
/// freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neb_fs_free_buf(buf: FfiBuf) {
    if !buf.ptr.is_null() {
        drop(unsafe { Box::from_raw(std::slice::from_raw_parts_mut(buf.ptr, buf.len)) });
    }
}

/// Frees an error message produced by this crate's functions.
///
/// # Safety
/// `ptr` must be a string produced by this crate, not previously freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neb_fs_free_error(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(unsafe { CString::from_raw(ptr) });
    }
}
