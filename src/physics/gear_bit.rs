//! `physics::gear_bit` (0x43C5B7): eax = ecx + 1, CF = bit eax of `g_carGearBits`
//! (0x71A40C, the work copy's +0x434; `bt` on memory, so a bit string). The callers pass
//! a gear (-1 .. 7, so bits 0 .. 8) and test CF (`jb`/`jnb`): a gear that cannot be
//! taken, by the look of `sub_43C20D` (a reading).
//!
//! In:  ecx the gear
//! Out: eax = ecx + 1, CF the bit; ZF as before (no caller reads it); the rest kept
//!
//! Callers (10): the gearbox under `sub_4454D2` (`sub_43C20D`, `sub_43C2AE` twice,
//! `sub_43C58E`, `sub_43C5A0`, `sub_43C5C2` twice, `sub_43C8AC`, `sub_4651B5`, unowned code
//! at 0x43C9AE). Refuses a bit outside 0 ..= 31 (past the dword; the original reads on
//! along the copy).

use crate::errors::Refused;
use crate::physics::car_store::CAR;
use crate::physics::flag_result::FlagResult;
use rc2k_core::physics::gearbox::GearBits;

/// The gear bits in the dword at `bits`.
pub unsafe fn bits_at(bits: u32) -> GearBits {
    GearBits(unsafe { (bits as *const u32).read_unaligned() })
}

/// The Rust version on the dword at `bits`.
// Converted: `rc2k_core::physics::gearbox::GearBits::test`.
pub unsafe fn gear_bit_at(bits: u32, gear: u32) -> Result<FlagResult, Refused> {
    let t = unsafe { bits_at(bits) }.test(gear)?;
    Ok(FlagResult {
        eax: t.bit,
        ecx: None,
        ebx: None,
        carry: t.barred,
        zero: None,
    })
}

/// The Rust version.
pub unsafe fn gear_bit(gear: u32) -> Result<FlagResult, Refused> {
    unsafe { gear_bit_at(CAR.lost_gear_bits.addr() as u32, gear) }
}
