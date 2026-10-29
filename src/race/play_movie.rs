//! `race::play_movie` (0x460776): "shaunplayrendermovie - in" logged, the clock restarted
//! (`scene::restart_clock`, `g_sceneClock` into `dword_613840`), the replay played
//! (`race::play_movie_loop`), "shaunplayrendermovie - out" logged.
//!
//! In:  nothing
//! Out: the registers as the play leaves them; no flags read
//!
//! Caller: `race::play_replay_range`.

use crate::errors::Refused;
use crate::physics::tables_store::PHYSICS;

const IN_TEXT: &std::ffi::CStr = c"shaunplayrendermovie - in"; // "shaunplayrendermovie - in"
const OUT_TEXT: &std::ffi::CStr = c"shaunplayrendermovie - out"; // "shaunplayrendermovie - out"

/// The Rust version. Reads no register; its caller (`race::play_replay_range`, which
/// keeps every register) reads none it leaves.
pub fn play_movie() -> Result<(), Refused> {
    crate::logfile::log_write::log_nul(&IN_TEXT, false, 1);
    crate::drive::race_parts::restart_clock();
    PHYSICS.race_clock_base.set(PHYSICS.scene_clock.get());
    crate::race::play_movie_loop::play_movie_loop()?;
    crate::logfile::log_write::log_nul(&OUT_TEXT, true, 1);
    Ok(())
}
