//! The game without the game: runs the ported code on `RAL.EXE`'s data (`standalone.rs`),
//! on SDL (`platform.rs`).
//!
//! Started in the game's folder: `rc2k.exe [path of RAL.EXE [path of the log]]`; the log
//! is `rc2k.log` there unless another is named. With the environment variable
//! `RC2K_FRAMES` set to a number it ends itself after that many frames and leaves the
//! last one in `rc2k_frame.raw`.

use std::path::Path;

fn main() {
    let game = std::env::args().nth(1).unwrap_or_else(|| "RAL.EXE".into());
    let log = std::env::args().nth(2).unwrap_or_else(|| "rc2k.log".into());
    let frames = std::env::var("RC2K_FRAMES")
        .ok()
        .and_then(|frames| frames.parse().ok())
        .unwrap_or(0);
    rc2k_hook::standalone::run(Path::new(&game), Path::new(&log), frames);
}
