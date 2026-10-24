//! `race::play_movie` (0x460776): "shaunplayrendermovie - in" logged, the clock restarted
//! (`scene::restart_clock`, `g_sceneClock` into `dword_613840`), the replay played
//! (`race::play_movie_loop`), "shaunplayrendermovie - out" logged.
//!
//! In:  nothing
//! Out: the registers as the play leaves them; no flags read
//!
//! Caller: `race::play_replay_range`. Its calls through `part`: 1 `logfile::log_write_l1`,
//! 2 `scene::restart_clock`, 3 `play_movie_loop`, 4 `logfile::log_write_l1_nl`; in the
//! game their Rust versions. Ungated (it plays a replay). Shadow mode runs the original
//! alone. Self-test: a copy whose calls go to stand-ins, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::physics::flow::a;
use crate::race::viewer::{get, put};

const SCENE_CLOCK: usize = 0x613830; // g_sceneClock
const CLOCK_BASE: usize = 0x613840; // dword_613840
const IN_TEXT: usize = 0x4DDF10; // "shaunplayrendermovie - in"
const OUT_TEXT: usize = 0x4DDF2A; // "shaunplayrendermovie - out"

/// The Rust version. Reads no register; its caller (`race::play_replay_range`, which
/// keeps every register) reads none it leaves.
pub fn play_movie() -> Result<(), Refused> {
    unsafe { crate::logfile::log_write::log_write(a(IN_TEXT) as *const u8, false, 1) };
    crate::drive::race_parts::restart_clock();
    put(CLOCK_BASE, get(SCENE_CLOCK));
    crate::race::play_movie_loop::play_movie_loop()?;
    unsafe { crate::logfile::log_write::log_write(a(OUT_TEXT) as *const u8, true, 1) };
    Ok(())
}
