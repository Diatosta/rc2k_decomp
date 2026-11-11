//! `physics::reset_point_blocked` (0x43F0DF): whether the checkpoint `reset_candidate` is no
//! place to put a reset car, as `physics::reset_car` ("cant reset car anywhere - waiting a
//! second") asks it of the other cars' blocks. Only when the block's `driver_kind` is 2 (a
//! computer car): blocked when the checkpoint is within 8 of the start or the end
//! (`g_checkpointCount` - 8), or within -8 .. 7 of the block's `reset_checkpoint`. The comparisons
//! take the sign of the difference, as here.
//!
//! Takes: the block
//!
//! Caller: `physics::reset_car` (six calls), when a car is reset onto the line: not every frame.

use crate::physics::tables_store::PhysicsTables;
use crate::race::vars_store::RaceVars;
use crate::session::Cell32;
use rc2k_core::physics::line_places;

fn at(a: &dyn Cell32) -> u32 {
    a.bits()
}

/// The candidate and count read at `candidate`, `count`; `block` the car block tried. Returns
/// the carry (blocked).
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

pub fn reset_point_blocked(tables: &PhysicsTables, vars: &RaceVars, block: u32) -> bool {
    reset_point_blocked_at(&tables.reset_candidate, &vars.checkpoint_count, block)
}
