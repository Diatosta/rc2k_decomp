#![allow(unsafe_code)]
//! The web build's start (`tools/web/`): the origin's private file storage (OPFS, through
//! Emscripten's WasmFS) mounted at `/opfs` before anything reads a file, and the game's folder
//! entered. The page (`tools/web/index.html`) unpacks the game folder's zip (or takes the picked
//! folder) into `/opfs/source`, and picks the folder each start: the installed copy
//! (`$HOME/.local/share/rally-championship`, `HOME` being `/opfs/home`) once there is one,
//! else the unpacked folder, where the program is the installer. A folder that is already an
//! installed game is put straight into the installed copy's place, and played without the
//! installer. It names the folder in `RC2K_DIR`.

unsafe extern "C" {
    fn wasmfs_create_opfs_backend() -> *mut core::ffi::c_void;
    fn wasmfs_create_directory(
        path: *const core::ffi::c_char,
        mode: u32,
        backend: *mut core::ffi::c_void,
    ) -> i32;
}

unsafe extern "C" {
    fn emscripten_force_exit(status: i32) -> !;
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

/// A folder as the storage reports it, with its mount point put back: below `/opfs` the file
/// system names the current folder from its own root (`//source` for `/opfs/source`), a path
/// that leads nowhere when the installer builds the files' paths from it.
pub fn with_mount(folder: &std::path::Path) -> std::path::PathBuf {
    if folder.starts_with(MOUNT) {
        folder.to_path_buf()
    } else {
        std::path::Path::new(MOUNT).join(folder.strip_prefix("/").unwrap_or(folder))
    }
}

/// The program ended with `code`: the runtime shut down, so that the page's `onExit` runs (the
/// page then shows that the game ended, or starts the installed game after the installer).
pub fn end_program(code: u32) -> ! {
    // SAFETY: Emscripten's documented call to end the program; it does not return.
    unsafe { emscripten_force_exit(code as i32) }
}
