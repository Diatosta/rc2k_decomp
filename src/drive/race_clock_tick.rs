//! `drive::race_clock_tick` (0x445554): the race's clock in tenths. `scene::restart_clock`
//! (the tick count into `g_sceneClock`), into `dword_61383C`; the last tick kept
//! (`dword_613840`) brought to within 1000 of it (by the sign of the `cmp`); the tenths
//! since (`div` by 10) into `dword_613844` and added back (x 10) to `dword_613840`.
//!
//! In:  nothing
//! Out: eax the tenths x 10, ebx 10, edx the product's high dword (0); the rest kept; no
//!      flags read
//!
//! Caller: `drive::race_step` (`sub_4454D2`).

use crate::race::owner_shared::Shared;
use rc2k_core::drive::clocks::RaceClock;

/// The Rust version after the clock's reading `now` (already in `g_sceneClock`).
// Converted: `rc2k_core::drive::clocks::RaceClock::tick`.
pub fn tick_from(shared: &Shared, now: u32) {
    let mut c = RaceClock {
        counted: shared.physics_tables.race_clock_base.get(),
        ..RaceClock::default()
    };
    c.tick(now);
    shared.physics_tables.race_clock_now.set(c.now);
    shared.physics_tables.race_tenths.set(c.tenths);
    shared.physics_tables.race_clock_base.set(c.counted);
}

/// The Rust version.
pub fn race_clock_tick(shared: &Shared) {
    let now = crate::scene::restart_clock::restart_clock(crate::system::tick_count::tick_count());
    tick_from(shared, now);
}
