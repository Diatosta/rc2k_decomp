//! `physics::lower_free_gear` (0x43C58E): from the gear in ebx down, the first lower gear
//! whose `g_carGearBits` bit (`physics::gear_bit`) is clear: ecx = ebx - 1, - 2, ..; CF
//! clear with ecx that gear, or CF set when ecx reaches 0 (neutral) first.
//!
//! In:  ebx the gear
//! Out: ecx the gear found (or 0), eax `gear_bit`'s (ecx + 1; kept when none was
//!      tested), CF; ZF not read by the callers (`jb`); the rest kept
//!
//! A loop target (0x43C590) is inside the first five bytes: `.private_copy`, its call
//! relocated. Callers (2): `sub_43C2AE` (0x43C397), `sub_43C521` (in `sub_43C2AE`), the
//! gearbox under `sub_4454D2`. Refuses what `gear_bit` refuses (a bit past the dword: from
//! ebx 0 or above 32 the search would run there), nothing written.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::physics::car_store::CAR;
use crate::physics::gear_bit::bits_at;

/// The Rust version on the bits at `bits`: the free lower gear (None when neutral is
/// reached first: CF). The eax it leaves is read by no caller, so none goes in.
// Converted: `rc2k_core::physics::gearbox::GearBits::lower_free`.
pub fn lower_free_gear_at(
    bits: &dyn crate::session::Cell32,
    gear: u32,
) -> Result<Option<u32>, Refused> {
    let s = { bits_at(bits) }.lower_free(gear, 0)?;
    Ok(s.found.then_some(s.gear))
}

/// The Rust version.
pub fn lower_free_gear(gear: u32) -> Result<Option<u32>, Refused> {
    lower_free_gear_at(&CAR.lost_gear_bits, gear)
}
