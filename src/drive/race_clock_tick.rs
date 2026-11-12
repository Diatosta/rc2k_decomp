//! `drive::race_clock_tick` (0x445554): the race's clock in tenths. `scene::restart_clock` (the
//! tick count into `g_sceneClock`), into `race_clock_now`; the last tick kept (`race_clock_base`)
//! brought to within 1000 of it (by the sign of the `cmp`); the tenths since (`div` by 10) into
//! `race_tenths` and added back (x 10) to `race_clock_base`.
//!
//! Caller: `drive::race_step` (0x4454D2).

use crate::race::owner_shared::RaceShared;
use rc2k_core::drive::clocks::RaceClock;

/// After the clock's reading `now` (already in `g_sceneClock`).
// Converted: `rc2k_core::drive::clocks::RaceClock::tick`.
pub fn tick_from(shared: &RaceShared, now: u32) {
    let mut c = RaceClock {
        counted: shared.physics_tables.get().race_clock_base.get(),
        ..RaceClock::default()
    };
    c.tick(now);
    shared.physics_tables.get().race_clock_now.set(c.now);
    shared.physics_tables.get().race_tenths.set(c.tenths);
    shared.physics_tables.get().race_clock_base.set(c.counted);
}

pub fn race_clock_tick(shared: &RaceShared) {
    let now =
        crate::scene::restart_clock::restart_clock(shared, crate::system::tick_count::tick_count());
    tick_from(shared, now);
}
