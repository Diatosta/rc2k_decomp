//! Whether the checkpoint `reset_candidate` is no place to put a reset car, as `physics::reset_car`
//! asks of the other cars' blocks. Only when the block's `driver_kind` is 2 (a computer car):
//! blocked when the checkpoint is within 8 of the start or the end (`checkpoint_count` - 8), or
//! within -8 .. 7 of the block's `reset_checkpoint`.
//!
//! Caller: `physics::reset_car` (six calls), when a car is reset onto the line: not every frame.

use crate::physics::tables_store::PhysicsTables;
use crate::race::vars_store::RaceVars;
use rc2k_core::physics::line_places;

/// Whether the reset candidate (`reset_candidate`) is blocked for the car block `block` (the
/// checkpoint count read only when asked). Returns true when blocked.
// Core: `rc2k_core::physics::line_places::reset_point_blocked`.
pub fn reset_point_blocked(
    tables: &PhysicsTables,
    vars: &RaceVars,
    block: &crate::physics::car_store::Car,
) -> bool {
    line_places::reset_point_blocked(
        block.driver_kind.get(),
        tables.reset_candidate.get(),
        || vars.checkpoint_count.get(),
        || block.reset_checkpoint.get(),
    )
    .carry()
}
