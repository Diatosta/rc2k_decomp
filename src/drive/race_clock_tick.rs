//! The race's clock in tenths: the scene clock restarted into `race_clock_now`; the last tick
//! kept (`race_clock_base`) brought to within 1000 of it; the tenths since (divided by 10) into
//! `race_tenths` and added back (x 10) to `race_clock_base`.

use crate::race::owner_shared::RaceShared;
use rc2k_core::drive::clocks::RaceClock;

fn tick_from(shared: &RaceShared, now: u32) {
    let mut c = RaceClock {
        counted: shared.run().race_clock_base,
        ..RaceClock::default()
    };
    c.tick(now);
    shared.run_mut().race_clock_now = c.now;
    shared.run_mut().race_tenths = c.tenths;
    shared.run_mut().race_clock_base = c.counted;
}

pub fn race_clock_tick(shared: &RaceShared, clock: &crate::clock::tick_count::PictureClock) {
    let now = crate::scene::restart_clock::restart_clock(
        shared,
        crate::clock::tick_count::tick_count(clock),
    );
    tick_from(shared, now);
}
