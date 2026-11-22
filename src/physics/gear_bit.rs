//! Whether gear + 1 is barred: the bit gear + 1 of `lost_gear_bits` (a bit string). The callers
//! pass a gear (-1 .. 7, so bits 0 .. 8) and test the result: a barred gear.
//!
//! Refuses a bit outside 0 ..= 31 (past the dword).

use crate::errors::Refused;
use crate::physics::car_store::Car;
use rc2k_core::physics::gearbox::GearBits;

/// The gear bits in the dword at `bits`.
pub(super) fn bits_at(bits: u32) -> GearBits {
    GearBits(bits)
}

/// Whether gear + 1 is barred in the dword at `bits`.
// Core: `rc2k_core::physics::gearbox::GearBits::test`.
pub(super) fn gear_bit_at(bits: u32, gear: i32) -> Result<bool, Refused> {
    Ok(bits_at(bits).test(gear)?.barred)
}

pub(super) fn gear_bit(car: &Car, gear: i32) -> Result<bool, Refused> {
    gear_bit_at(car.lost_gear_bits, gear)
}
