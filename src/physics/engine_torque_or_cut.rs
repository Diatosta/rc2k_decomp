//! `physics::engine_torque_or_cut` (0x43CB7D): at or past the engine's top the cut:
//! +0x1F0 (`unk_71A1C8`) 5 and the torque `flt_614748` 0; below it (an unordered compare
//! counts as below: C0, `jb`; IDA 2026-10-21), while +0x1F0 counts down the torque stays
//! 0; else `physics::engine_torque`.
//!
//! In:  nothing
//! Out: as `engine_torque` leaves them when it ran; no flags read
//!
//! Caller: `physics::drive_force`.

use crate::physics::car_store::CAR;
use crate::physics::engine_torque::{below, engine_torque};
use crate::physics::tables_store::PHYSICS;

/// The Rust version (the exact one, for `physics::drive_force`).
pub unsafe fn engine_torque_or_cut() {
    if !below(CAR.engine_speed.bits(), CAR.engine_speed_limit.bits()) {
        CAR.rev_cut_count.set(5);
    } else if CAR.rev_cut_count.get() != 0 {
        CAR.rev_cut_count.set(
            crate::physics::car_store::CAR
                .rev_cut_count
                .get()
                .wrapping_sub(1),
        );
    } else {
        unsafe { engine_torque() };
        return;
    }
    PHYSICS.engine_force.set_bits(0);
}
