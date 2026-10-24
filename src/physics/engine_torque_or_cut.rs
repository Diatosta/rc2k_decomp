//! `physics::engine_torque_or_cut` (0x43CB7D): at or past the engine's top the cut:
//! +0x1F0 (`unk_71A1C8`) 5 and the torque `flt_614748` 0; below it (an unordered compare
//! counts as below: C0, `jb`; IDA 2026-10-21), while +0x1F0 counts down the torque stays
//! 0; else `physics::engine_torque`.
//!
//! In:  nothing
//! Out: as `engine_torque` leaves them when it ran; no flags read
//!
//! Caller: `physics::drive_force`.

use crate::physics::engine_torque::{below, engine_torque, g, p};

/// The Rust version (the exact one, for `physics::drive_force`).
pub unsafe fn engine_torque_or_cut() {
    if !below(g(0x71A1B0), g(0x71A1B4)) {
        p(0x71A1C8, 5);
    } else if g(0x71A1C8) != 0 {
        p(0x71A1C8, g(0x71A1C8).wrapping_sub(1));
    } else {
        unsafe { engine_torque() };
        return;
    }
    p(0x614748, 0);
}
