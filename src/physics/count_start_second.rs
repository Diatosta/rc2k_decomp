//! `physics::count_start_second` (0x436AC4): `g_car` +0x88 (`unk_71A060`) up by one when
//! +0x26C (`unk_71A244`) is -1 or 0, the start count +0x1C is 0, the timer +0x14 less
//! 0x320 is not below 0 (by its sign) and +0xA8 (`unk_71A080`) is 0. A reading: time
//! counted once the start is behind (`physics::false_start`'s fields).
//!
//! In:  nothing
//! Out: eax the timer less 0x320 when it got that far; the rest kept; no flags read
//!
//! Caller: `sub_436790` (four calls).

use crate::physics::car_store::CAR;
use crate::session::Cell32;
use rc2k_core::physics::start::CarStart;

fn game() -> Vec<&'static dyn Cell32> {
    vec![
        &CAR.tyre_change_state as &dyn Cell32,
        &CAR.false_start_count,
        &CAR.run_ticks,
        &CAR.finish_flag,
        &CAR.stage_time,
    ]
}

/// The Rust version on the variables' cells.
// Converted: `rc2k_core::physics::start::CarStart::count_start_second`.
fn core(v: &[&dyn Cell32]) {
    let was = CarStart {
        state: v[0].bits(),
        count: v[1].bits(),
        timer: v[2].bits(),
        finished: v[3].bits(),
        time: v[4].bits(),
        ..CarStart::default()
    };
    let mut c = was;
    c.count_start_second();
    crate::physics::cells::set_changed(v[4], was.time, c.time);
}

/// The Rust version.
pub fn count_start_second() {
    core(&game())
}
