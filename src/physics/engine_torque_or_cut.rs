//! `physics::engine_torque_or_cut` (0x43CB7D): at or past the engine's top the cut:
//! +0x1F0 (`unk_71A1C8`) 5 and the torque `flt_614748` 0; below it (an unordered compare
//! counts as below: C0, `jb`; IDA 2026-10-21), while +0x1F0 counts down the torque stays
//! 0; else `physics::engine_torque`.
//!
//! In:  nothing
//! Out: as `engine_torque` leaves them when it ran; no flags read
//!
//! Caller: `physics::drive_force`.
//! Self-test: `physics::engine_torque`'s (`test_engine_floats`), the converted version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::physics::engine_torque::{below, engine_torque, g, p};
use crate::screens::harness::Left;

/// The Rust version (the exact one, for `physics::drive_force`).
pub unsafe fn engine_torque_or_cut() -> Left {
    if !below(g(0x71A1B0), g(0x71A1B4)) {
        p(0x71A1C8, 5);
    } else if g(0x71A1C8) != 0 {
        p(0x71A1C8, g(0x71A1C8).wrapping_sub(1));
    } else {
        return unsafe { engine_torque() };
    }
    p(0x614748, 0);
    Left::nothing()
}
