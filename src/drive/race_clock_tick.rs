//! The race's clock in tenths: the scene clock restarted into `race_clock_now`; the last tick
//! kept (`race_clock_base`) brought to within 1000 of it; the tenths since (divided by 10) into
//! `race_tenths` and added back (x 10) to `race_clock_base`.

use crate::race::owner_shared::RaceShared;
use rc2k_core::drive::clocks::RaceClock;

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

pub fn race_clock_tick(shared: &RaceShared, clock: &crate::system::tick_count::PictureClock) {
    let now = crate::scene::restart_clock::restart_clock(
        shared,
        crate::system::tick_count::tick_count(clock),
    );
    tick_from(shared, now);
}
