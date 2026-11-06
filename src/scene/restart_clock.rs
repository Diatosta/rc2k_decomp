//! `scene::restart_clock` (0x450381): `g_sceneClock` = the tick count.
//!
//! In:  nothing
//! Out: eax = the tick count; every other register preserved
//!
//! 21 callers (`race::load_stage` and the race's code). The registers above are the original's;
//! a call by address goes through `routes.rs`, which maps them.

/// The clock set to `now`; eax.
pub fn restart_clock(now: u32) -> u32 {
    crate::race::owner::race()
        .shared
        .physics_tables
        .scene_clock
        .set_bits(now);
    now
}
