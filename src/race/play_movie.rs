//! `race::play_movie` (0x460776): "shaunplayrendermovie - in" logged, the clock restarted
//! (`scene::restart_clock`, `g_sceneClock` into `dword_613840`), the replay played
//! (`race::play_movie_loop`), "shaunplayrendermovie - out" logged.
//!
//! In:  nothing
//! Out: the registers as the play leaves them; no flags read
//!
//! Caller: `race::play_replay_range`.

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
