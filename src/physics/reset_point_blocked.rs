//! `physics::reset_point_blocked` (0x43F0DF): whether the checkpoint `dword_614C50` is no
//! place to put a reset car, as `sub_43EFEF` ("cant reset car anywhere - waiting a
//! second") asks it of the other cars' blocks (esi 0x7103CC, 0x7113AC, 0x71238C, ...).
//! Only when the block's +0x25C is 2 (by the look of it, a car in the stage; a reading):
//! CF when the checkpoint is within 8 of the start or the end (`g_checkpointCount` - 8),
//! or within -8 .. 7 of the block's +0x94 (its checkpoint). The comparisons take the sign
//! of the difference (`js`/`jns`), as here.
//!
//! In:  esi the block
//! Out: CF; eax and ebx as the path leaves them (see `reset_point_blocked_at`); the rest
//!      kept. ZF is not read (`jb`).
//!
//! Caller: `sub_43EFEF` (six calls), when a car is reset onto the line: not every frame.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::race::owner::race;
use crate::session::Cell32;
use rc2k_core::physics::line_places;

fn at(a: &dyn Cell32) -> u32 {
    a.bits()
}

/// The Rust core: the candidate and count read at `candidate`, `count`; `block` the car
/// block tried (esi). Returns the carry (blocked); the eax and ebx it leaves are read by
/// none of its callers.
// Converted: `rc2k_core::physics::line_places::reset_point_blocked`.
pub fn reset_point_blocked_at(candidate: &dyn Cell32, count: &dyn Cell32, block: u32) -> bool {
    let block = crate::physics::car_store::Car::at(block);
    line_places::reset_point_blocked(
        block.driver_kind.get(),
        at(candidate),
        || at(count),
        || block.reset_checkpoint.get(),
    )
    .carry()
}

/// The Rust version.
pub fn reset_point_blocked(block: u32) -> bool {
    let race = race();
    reset_point_blocked_at(
        &race.shared.physics_tables.reset_candidate,
        &race.shared.vars.checkpoint_count,
        block,
    )
}
