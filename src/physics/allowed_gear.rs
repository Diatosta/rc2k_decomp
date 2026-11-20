//! The gear the player asked for, or 0 when the gearbox refuses it: 0 when the engine is dead
//! (its damage word holds the bits of 100.0), when `physics::gear_bit` bars a gear of 1 or more,
//! and when `engine_power_factor` is not set.
//!
//! Refuses a gear whose bit is past 31 (as `gear_bit`).

use crate::errors::Refused;
use crate::memory::session::{PlainDword, PlainFloat};
use crate::physics::car_store::Car;
use crate::physics::gear_bit::bits_at;
use crate::race::numbers_store::Numbers;

/// On the engine damage word, the engine power factor and the gear bits at these places.
// Core: `rc2k_core::physics::gearbox::allowed_gear`.
fn allowed_gear_at(
    numbers: &Numbers,
    locked: &PlainFloat,
    open: &PlainFloat,
    bits: &PlainDword,
    eax: i32,
) -> Result<i32, Refused> {
    // The words are compared: the engine damage is exactly 100.0's bits, not a float compare.
    let locked = locked.get() == numbers.hundred.get();
    Ok(rc2k_core::physics::gearbox::allowed_gear(
        locked,
        eax,
        || bits_at(bits),
        || open.bits(),
    )?)
}

pub fn allowed_gear(car: &Car, numbers: &Numbers, eax: i32) -> Result<i32, Refused> {
    allowed_gear_at(
        numbers,
        &car.engine_damage,
        &car.engine_power_factor,
        &car.lost_gear_bits,
        eax,
    )
}
