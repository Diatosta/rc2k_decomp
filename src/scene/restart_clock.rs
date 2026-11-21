//! `scene::restart_clock`: the scene clock = the tick count, which it returns. 21 callers
//! (`race::load_stage` and the race's code).

use crate::race::owner_shared::RaceShared;

/// The clock set to `now`.
pub fn restart_clock(shared: &RaceShared, now: u32) -> u32 {
    shared.run_mut().scene_clock = now;
    now
}
