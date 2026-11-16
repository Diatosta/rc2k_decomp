#![allow(unsafe_code)]
//! The web build's start (`tools/web/`): the origin's private file storage (OPFS, through
//! Emscripten's WasmFS) mounted at `/opfs` before anything reads a file, and the game's folder
//! entered. The page (`tools/web/index.html`) unpacks the game folder's zip into
//! `/opfs/source` and picks the folder each start: the installed copy
//! (`$HOME/.local/share/rally-championship`, `HOME` being `/opfs/home`) once there is one,
//! else the unpacked folder, where the program is the installer. It names it in `RC2K_DIR`.

unsafe extern "C" {
    fn wasmfs_create_opfs_backend() -> *mut core::ffi::c_void;
    fn wasmfs_create_directory(
        path: *const core::ffi::c_char,
        mode: u32,
        backend: *mut core::ffi::c_void,
    ) -> i32;
}

/// Where the storage is mounted.
pub const MOUNT: &str = "/opfs";

/// The storage mounted and the game's folder entered; a line on the console for each step
/// (the log is not open yet).
pub fn enter_game_folder() {
    // SAFETY: WasmFS's documented calls, on the program's own thread (a worker: the OPFS
    // backend may not run on the page's thread), with a constant NUL-ended path.
    let made = unsafe {
        let backend = wasmfs_create_opfs_backend();
        if backend.is_null() {
            -1
        } else {
            wasmfs_create_directory(c"/opfs".as_ptr(), 0o777, backend)
        }
    };
    eprintln!("web: the storage mounted at {MOUNT}: {made}");
    let dir = std::env::var("RC2K_DIR").unwrap_or_else(|_| format!("{MOUNT}/source"));
    let _ = std::fs::create_dir_all(&dir);
    match std::env::set_current_dir(&dir) {
        Ok(()) => eprintln!("web: the game's folder is {dir}"),
        Err(e) => eprintln!("web: the game's folder {dir} cannot be entered: {e}"),
    }
}
