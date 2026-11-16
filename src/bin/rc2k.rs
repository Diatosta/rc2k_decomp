//! The game's program: runs the game (`standalone.rs`) on SDL (`platform/mod.rs`).
//!
//! Started in the game's folder: `rc2k [log] [switches]`; the log is `rc2k.log` there unless
//! another is named. The game's own switches (`-a`, `-c`, `-p`, ..) are read off the command
//! line by the game itself (`system::read_command_line`); a word naming a `.exe` is passed
//! over. With the environment variable `RC2K_FRAMES` set to a number it ends itself after that
//! many frames and leaves the last one in `rc2k_frame.raw`. `--stage` and its kin start a race
//! straight away (`menu::launch`). `--help` lists every switch (`rc2k_core::cli`); a `--` switch
//! the program does not know ends it before it starts.
//!
//! A windowed program on Windows: no console, everything goes to the log.

#![windows_subsystem = "windows"]

use std::path::Path;

fn main() {
    #[cfg(target_os = "emscripten")]
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
