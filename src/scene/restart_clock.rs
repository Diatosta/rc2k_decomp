//! `scene::restart_clock` (0x450381): `g_sceneClock` = the tick count.
//!
//! Returns: the tick count
//!
//! 21 callers (`race::load_stage` and the race's code). The registers above are the original's;
//! a call by address goes through `routes.rs`, which maps them.

use crate::race::owner_shared::RaceShared;

/// The clock set to `now`; eax.
pub fn restart_clock(shared: &RaceShared, now: u32) -> u32 {
    shared.physics_tables.scene_clock.set(now);
    now
}
