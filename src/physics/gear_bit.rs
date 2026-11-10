//! `physics::gear_bit` (0x43C5B7): eax = ecx + 1, CF = bit eax of `g_carGearBits`
//! (0x71A40C, the work copy's +0x434; `bt` on memory, so a bit string). The callers pass
//! a gear (-1 .. 7, so bits 0 .. 8) and test CF (`jb`/`jnb`): a gear that is barred
//! (`lost_gear_bits`, read by `physics::change_gear`).
//!
//! Takes: the gear
//!
//! Callers (10): the gearbox under `drive::race_step` (0x43C20D, `physics::manual_gearbox` twice,
//! `physics::lower_free_gear`, `physics::higher_free_gear`, `physics::auto_gearbox` twice,
//! `physics::pick_start_gear`, `physics::allowed_gear`, unowned code at 0x43C9AE). Refuses a bit
//! outside 0 ..= 31 (past the dword; the original reads on along the copy).

use crate::errors::Refused;
use crate::physics::flag_result::FlagResult;
use crate::race::owner_shared::Shared;
use rc2k_core::physics::gearbox::GearBits;

/// The gear bits in the dword at `bits`.
pub fn bits_at(bits: &dyn crate::session::Cell32) -> GearBits {
    GearBits(bits.bits())
}

/// On the dword at `bits`.
// Converted: `rc2k_core::physics::gearbox::GearBits::test`.
pub fn gear_bit_at(bits: &dyn crate::session::Cell32, gear: u32) -> Result<FlagResult, Refused> {
    let t = { bits_at(bits) }.test(gear)?;
    Ok(FlagResult {
        eax: t.bit,
        ecx: None,
        ebx: None,
        carry: t.barred,
        zero: None,
    })
}

pub fn gear_bit(shared: &Shared, gear: u32) -> Result<FlagResult, Refused> {
    gear_bit_at(&shared.car.lost_gear_bits, gear)
}
