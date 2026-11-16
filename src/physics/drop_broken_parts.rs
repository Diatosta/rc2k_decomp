//! Seven parts of the work car checked against their limits: when the part's damage word is not
//! 0x80000000 and not below (unsigned) its entry of `part_break_limits`,
//! `physics::fill_slot_on_bit` runs with bit 9, 0xD, 0xE, 0x10, 0x11, 0xC, 0xF (the first only
//! with `marks_mask` set). A reading: damaged parts falling off.
//!
//! Caller: `physics::car_physics_step`, every physics step. Refuses what `fill_slot_on_bit`
//! refuses; what was written is put back.

use crate::errors::Refused;
use crate::physics::car_store::Car;
use crate::physics::fill_slot_on_bit::fill_slot_on_bit;
use crate::race::numbers_store::Numbers;
use crate::race::work_store::RaceWork;
use rc2k_core::physics::parts::{BROKEN_BITS, breaks, broken_checked};

// Core: `rc2k_core::physics::parts::{BROKEN_BITS, broken_checked, breaks}`.
pub fn drop_broken_parts(
    numbers: &Numbers,
    work: &RaceWork,
    car: &Car,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    for (k, &bit) in BROKEN_BITS.iter().enumerate() {
        if !broken_checked(k, || scene.lights.work.get().marks_mask.get() != 0) {
            continue;
        }
        if !breaks(car.part_damage_limits[k].get(), || {
            work.part_break_limits[k].get()
        }) {
            continue;
        }
        fill_slot_on_bit(numbers, work, car, bit, scene)?;
    }
    Ok(())
}
