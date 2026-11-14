//! Whether gear + 1 is barred: the bit gear + 1 of `lost_gear_bits` (a bit string). The callers
//! pass a gear (-1 .. 7, so bits 0 .. 8) and test the result: a barred gear.
//!
//! Callers: the gearbox under `drive::race_step` (`physics::change_gear`, `physics::manual_gearbox`
//! twice, `physics::lower_free_gear`, `physics::higher_free_gear`, `physics::auto_gearbox` twice,
//! `physics::pick_start_gear` and `physics::allowed_gear`). Refuses a bit outside 0 ..= 31 (past
//! the dword).

use crate::errors::Refused;
use crate::physics::car_store::Car;
use crate::physics::flag_result::FlagResult;
use crate::session::PlainDword;
use rc2k_core::physics::gearbox::GearBits;

/// The gear bits in the dword at `bits`.
pub fn bits_at(bits: &PlainDword) -> GearBits {
    GearBits(bits.get())
}

/// On the dword at `bits`.
// Core: `rc2k_core::physics::gearbox::GearBits::test`.
pub fn gear_bit_at(bits: &PlainDword, gear: i32) -> Result<FlagResult, Refused> {
    let t = { bits_at(bits) }.test(gear)?;
    Ok(FlagResult {
        eax: t.bit,
        ecx: None,
        ebx: None,
        carry: t.barred,
        zero: None,
    })
}

pub fn gear_bit(car: &Car, gear: i32) -> Result<FlagResult, Refused> {
    gear_bit_at(&car.lost_gear_bits, gear)
}
