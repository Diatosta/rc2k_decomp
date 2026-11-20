//! The game's program: runs the game (`standalone.rs`) on SDL (`platform/mod.rs`).
//!
//! Started in the game's folder: `rc2k [log] [switches]`; the log is `rc2k.log` there unless
//! another is named. The game's own switches (`-a`, `-c`, `-u`, ..) are read off the command
//! line by the game itself (`system::read_command_line`); a word naming a `.exe` is passed
//! over. With the environment variable `RC2K_FRAMES` set to a number it ends itself after that
//! many frames and leaves the last one in `rc2k_frame.raw`. `--stage` and its kin start a race
//! straight away (`menu::launch`). `--help` lists every switch (`rc2k_core::cli`); a `--` switch
//! the program does not know ends it before it starts.
//!
//! A windowed program on Windows: no console, everything goes to the log.

#![windows_subsystem = "windows"]

use std::path::Path;

/// On iOS the program is an application: SDL starts the system's application loop and calls
/// `game` from its first event, on the main thread.
#[cfg(target_os = "ios")]
fn main() {
    use std::ffi::CString;
    let mut texts: Vec<CString> = std::env::args()
        .filter_map(|a| CString::new(a).ok())
        .collect();
    let mut pointers: Vec<*mut std::ffi::c_char> =
        texts.iter_mut().map(|t| t.as_ptr().cast_mut()).collect();
    pointers.push(std::ptr::null_mut());
    unsafe extern "C" fn entry(
        _: std::ffi::c_int,
        _: *mut *mut std::ffi::c_char,
    ) -> std::ffi::c_int {
        game();
        0
    }
    // SAFETY: `pointers` is a null-ended list of the NUL-ended `texts`, both alive for the call
    // (which does not return until the application ends); `entry` has SDL's function type.
    let code = unsafe {
        sdl3_sys::main::SDL_RunApp(
            (pointers.len() - 1) as std::ffi::c_int,
            pointers.as_mut_ptr(),
            Some(entry),
            std::ptr::null_mut(),
        )
    };
    std::process::exit(code);
}

#[cfg(not(target_os = "ios"))]
fn main() {
    game();
}

fn game() {
    #[cfg(web)]
    rc2k_decomp::web::enter_game_folder();
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(code) = rc2k_decomp::standalone::answer_command_line(&args) {
        std::process::exit(code);
    }
    let log = rc2k_core::menu::launch::log_path(&args).unwrap_or_else(|| "rc2k.log".into());
    let frames = std::env::var("RC2K_FRAMES")
        .ok()
        .and_then(|frames| frames.parse().ok())
        .unwrap_or(0);
    rc2k_decomp::standalone::run(Path::new(&log), frames);
}
