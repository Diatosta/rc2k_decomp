//! `physics::apply_wheel_torques` (0x43F694): `physics::apply_torques` on the 0x17
//! records at `unk_614770` (the ones `physics::apply_wheel_forces` pushes with).
//!
//! In:  nothing
//! Out: ebx the count, edi the first record; the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B822, after `physics::apply_wheel_forces`, before
//! `physics::move_car`), every physics step.

use crate::errors::Refused;
use crate::physics::apply_torques::apply_torques;
use crate::physics::tables_store::PHYSICS;
const COUNT: u32 = 0x17;

/// The Rust version.
pub unsafe fn apply_wheel_torques() -> Result<(), Refused> {
    let first = (PHYSICS.forces.as_ptr() as usize) as u32;
    unsafe { apply_torques(first, COUNT) }
}
