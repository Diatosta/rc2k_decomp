//! `physics::engine_torque_or_cut` (0x43CB7D): at or past the engine's top the cut:
//! +0x1F0 (`unk_71A1C8`) 5 and the torque `flt_614748` 0; below it (an unordered compare
//! counts as below: C0, `jb`; IDA 2026-10-21), while +0x1F0 counts down the torque stays
//! 0; else `physics::engine_torque`.
//!
//! In:  nothing
//! Out: as `engine_torque` leaves them when it ran; no flags read
//!
//! Caller: `physics::drive_force`.

use crate::physics::engine_torque::{below, engine_torque};
use crate::race::owner::Race;

/// The Rust version (the exact one, for `physics::drive_force`).
pub fn engine_torque_or_cut(race: &Race) {
    let shared = &race.shared;
    if !below(
        shared.car.engine_speed.bits(),
        shared.car.engine_speed_limit.bits(),
    ) {
        shared.car.rev_cut_count.set(5);
    } else if shared.car.rev_cut_count.get() != 0 {
        shared
            .car
            .rev_cut_count
            .set(shared.car.rev_cut_count.get().wrapping_sub(1));
    } else {
        engine_torque(race);
        return;
    }
    shared.physics_tables.engine_force.set_bits(0);
}
