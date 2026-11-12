//! The game without the game: runs the ported code on `RAL.EXE`'s data (`standalone.rs`),
//! on SDL (`platform.rs`).
//!
//! Started in the game's folder: `rc2k.exe [log] [switches]`; the log is `rc2k.log` there
//! unless another is named. The game's data built into `RAL.EXE` is the program's own (its
//! statics' first values): RAL.EXE is not needed. The game's switches (`-a`, `-c`, `-p`, ..)
//! are read off the command line by the game itself (`system::read_command_line`); a word
//! naming a `.exe` is passed over (the old `rc2k.exe RAL.EXE [log]`). With the environment
//! variable `RC2K_FRAMES` set to a number it ends itself after that many frames and leaves
//! the last one in `rc2k_frame.raw`.
//!
//! A windowed program (no console: everything goes to the log; owner, 2026-10-19).

#![windows_subsystem = "windows"]

use std::path::Path;

fn main() {
    #[cfg(target_os = "emscripten")]
    rc2k_decomp::web::enter_game_folder();
    let log = std::env::args()
        .skip(1)
        .find(|a| !a.starts_with('-') && !a.to_ascii_lowercase().ends_with(".exe"))
        .unwrap_or_else(|| "rc2k.log".into());
    let frames = std::env::var("RC2K_FRAMES")
        .ok()
        .and_then(|frames| frames.parse().ok())
        .unwrap_or(0);
    rc2k_decomp::standalone::run(Path::new(&log), frames);
}
