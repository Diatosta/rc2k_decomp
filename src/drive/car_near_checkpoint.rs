//! `drive::car_near_checkpoint` (0x4375F2): whether a car (esi) in the stage (+0x25C not
//! 0) is within 3 checkpoints of eax: CF when |+0x98 - eax| (`neg` of a negative) less 4
//! has its sign set.
//!
//! Takes: a car block, a checkpoint
//! Out: CF; ebx the distance when +0x25C is not 0; the rest kept (ZF not read: `jb`)
//!
//! Caller: `drive::checkpoint_taken` (four cars).

use crate::physics::car_store::Car;
use rc2k_core::physics::line_places::car_near;

/// The Rust core: whether the car block `car` is near `checkpoint` (eax; the CF).
/// The distance it leaves in ebx is read by no caller (`checkpoint_taken` keeps ebx).
// Converted: `rc2k_core::physics::line_places::car_near`.
pub fn near_at(car: &Car, checkpoint: u32) -> bool {
    car_near(
        car.driver_kind.bits() != 0,
        || car.checkpoint.bits(),
        checkpoint,
    )
    .is_some_and(|n| n.1)
}
