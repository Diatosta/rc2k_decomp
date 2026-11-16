//! Whether gear + 1 is barred: the bit gear + 1 of `lost_gear_bits` (a bit string). The callers
//! pass a gear (-1 .. 7, so bits 0 .. 8) and test the result: a barred gear.
//!
//! Refuses a bit outside 0 ..= 31 (past the dword).

use crate::errors::Refused;
use crate::memory::session::PlainDword;
use crate::physics::car_store::Car;
use rc2k_core::physics::gearbox::GearBits;

/// The gear bits in the dword at `bits`.
pub fn bits_at(bits: &PlainDword) -> GearBits {
    GearBits(bits.get())
}

/// Whether gear + 1 is barred in the dword at `bits`.
// Core: `rc2k_core::physics::gearbox::GearBits::test`.
pub fn gear_bit_at(bits: &PlainDword, gear: i32) -> Result<bool, Refused> {
    Ok(bits_at(bits).test(gear)?.barred)
}

pub fn gear_bit(car: &Car, gear: i32) -> Result<bool, Refused> {
    gear_bit_at(&car.lost_gear_bits, gear)
}
