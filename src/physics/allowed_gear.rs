//! `physics::allowed_gear` (0x4651B5): the gear the player asked for, or 0. Gives 0 when the
//! engine damage word (`engine_damage`) holds the bits of 100.0 (the engine is dead, compared as
//! a dword); for a gear of 1 or more also 0 when `physics::gear_bit` reports it as barred; and
//! finally 0 unless `engine_power_factor` is set. A reading: the gear asked for, unless the
//! gearbox refuses it.
//!
//! Takes: the gear
//!
//! Caller: `physics::take_asked_gear`. Refuses what `gear_bit` refuses (a bit past 31).

use crate::errors::Refused;
use crate::physics::car_store::Car;
use crate::physics::gear_bit::bits_at;
use crate::race::numbers_store::Numbers;
use crate::session::{PlainDword, PlainFloat};

/// On the word +0x2B8, the word +0x3F4 and the gear bits at these places.
// Converted: `rc2k_core::physics::gearbox::allowed_gear`.
pub fn allowed_gear_at(
    numbers: &Numbers,
    locked: &PlainFloat,
    open: &PlainFloat,
    bits: &PlainDword,
    eax: u32,
) -> Result<u32, Refused> {
    // The game compares the words: the engine damage is exactly 100.0's bits, not a float compare.
    let locked = locked.bits() == numbers.hundred.bits();
    Ok(rc2k_core::physics::gearbox::allowed_gear(
        locked,
        eax,
        || bits_at(bits),
        || open.bits(),
    )?)
}

pub fn allowed_gear(car: &Car, numbers: &Numbers, eax: u32) -> Result<u32, Refused> {
    allowed_gear_at(
        numbers,
        &car.engine_damage,
        &car.engine_power_factor,
        &car.lost_gear_bits,
        eax,
    )
}
