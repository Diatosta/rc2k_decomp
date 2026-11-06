//! `physics::higher_free_gear` (0x43C5A0): from the gear in ebx up, the first higher gear
//! whose `g_carGearBits` bit (`physics::gear_bit`) is clear: ecx = ebx + 1, + 2, .. up to
//! `g_gearCount` (0x614C10, unsigned); CF clear with ecx that gear, or CF set with ecx one
//! past the count when none is.
//!
//! In:  ebx the gear
//! Out: ecx, eax `gear_bit`'s (ecx + 1 of the last tested), CF; ZF not read by the
//!      caller (`jb`); the rest kept
//!
//! A loop target (0x43C5A3) is inside the first five bytes: `.private_copy`, its call
//! relocated. Caller: `sub_43C2AE` (0x43C365), the gearbox under `sub_4454D2`. Refuses what
//! `gear_bit` refuses (a bit past the dword: a count of 31 or more lets the search run
//! there), nothing written.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::physics::gear_bit::bits_at;
use crate::race::owner::Race;

/// The Rust version on the bits at `bits` and the count `count`.
// Converted: `rc2k_core::physics::gearbox::GearBits::higher_free`.
pub fn higher_free_gear_at(
    bits: &dyn crate::session::Cell32,
    count: u32,
    gear: u32,
) -> Result<Option<u32>, Refused> {
    let s = { bits_at(bits) }.higher_free(gear, count)?;
    Ok(s.found.then_some(s.gear))
}

/// The Rust version.
pub fn higher_free_gear(race: &Race, gear: u32) -> Result<Option<u32>, Refused> {
    higher_free_gear_at(
        &race.shared.car.lost_gear_bits,
        race.shared.physics_tables.gear_count.bits(),
        gear,
    )
}
