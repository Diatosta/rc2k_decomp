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
    // The page's thread's side (`tools/web/library.js`): each call waits for the page.
    fn rc2k_web_sizes(out: *mut i32);
    fn rc2k_web_refresh() -> f64;
    fn rc2k_web_set_full_screen(on: i32) -> i32;
    fn rc2k_web_is_full_screen() -> i32;
    fn rc2k_web_download(
        name: *const core::ffi::c_char,
        data: *const u8,
        len: i32,
        mime: *const core::ffi::c_char,
    );
}

unsafe extern "C" {
    fn emscripten_force_exit(status: i32) -> !;
    fn emscripten_current_thread_process_queued_calls();
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

/// The browser window's and the screen's sizes in device pixels.
pub struct Sizes {
    pub window: (u32, u32),
    pub screen: (u32, u32),
}

/// The page's window and screen as they are now, `None` when the page reports no size.
pub fn sizes() -> Option<Sizes> {
    let mut numbers = [0i32; 8];
    // SAFETY: the page's function writes six numbers into the eight there are room for.
    unsafe { rc2k_web_sizes(numbers.as_mut_ptr()) };
    let size = |w: i32, h: i32| (w > 0 && h > 0).then_some((w as u32, h as u32));
    Some(Sizes {
        window: size(numbers[0], numbers[1])?,
        screen: size(numbers[2], numbers[3])?,
    })
}

/// The display's refreshes a second as the page measured them (`None` before it has).
pub fn display_refresh() -> Option<f64> {
    // SAFETY: a call into the page that takes nothing and answers a number.
    let rate = unsafe { rc2k_web_refresh() };
    (rate.is_finite() && rate >= 1.0).then_some(rate)
}

/// The page put into the browser's full screen (`true`) or back; false when the browser did
/// not take the request (it wants a key or a click behind it).
pub fn set_full_screen(on: bool) -> bool {
    // SAFETY: a call into the page that takes a flag and answers 0 or 1.
    unsafe { rc2k_web_set_full_screen(i32::from(on)) != 0 }
}

/// Whether the page is in the browser's full screen now.
pub fn is_full_screen() -> bool {
    // SAFETY: a call into the page that takes nothing and answers 0 or 1.
    unsafe { rc2k_web_is_full_screen() != 0 }
}

/// Whether the player has just left the browser's full screen (with Esc or F11): the window
/// mode kept then goes back to a window, as the graphics page shows. Asks the page once in 30
/// calls (the events' pump runs for every frame).
pub fn left_full_screen() -> bool {
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering::Relaxed};
    static CALLS: AtomicU32 = AtomicU32::new(0);
    static WAS_FULL: AtomicBool = AtomicBool::new(false);
    if CALLS.fetch_add(1, Relaxed) % 30 != 0 {
        return false;
    }
    let now = is_full_screen();
    WAS_FULL.swap(now, Relaxed) && !now
}

/// `bytes` handed to the browser as a download named `name` (a file's last part: folders are
/// cut off), of the media type `mime`. A name with a NUL in it, or bytes too many for the page's
/// call, are not sent.
pub fn download(name: &str, bytes: &[u8], mime: &str) {
    let last = name.rsplit(['/', '\\']).next().unwrap_or(name);
    let (Ok(name), Ok(mime), Ok(len)) = (
        std::ffi::CString::new(last),
        std::ffi::CString::new(mime),
        i32::try_from(bytes.len()),
    ) else {
        return;
    };
    // SAFETY: the page's function copies `len` bytes from `bytes` and reads the two NUL-ended
    // texts before it returns; all three outlive the call.
    unsafe { rc2k_web_download(name.as_ptr(), bytes.as_ptr(), len, mime.as_ptr()) };
}

/// The calls the page's thread has queued for this thread run now. The browser's events (keys,
/// the mouse, the window's focus) reach SDL as calls queued for the thread that made its
/// window; a thread that never waits on a lock or a futex runs them only when asked.
pub fn run_queued_calls() {
    // SAFETY: Emscripten's documented call, made on this thread's own queue; it takes nothing.
    unsafe { emscripten_current_thread_process_queued_calls() };
}
