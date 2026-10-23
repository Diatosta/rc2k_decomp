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
//! gearbox under `sub_4454D2`.
//! Refuses what `gear_bit` refuses (a bit past the dword: from ebx 0 or above 32 the
//! search would run there), nothing written.
//! Shadow mode: `physics/flag_result.rs`.
//! Self-test: a copy of the original whose call goes to a copy of `gear_bit` on a test
//! dword, gears 1 .. 9 on several bit patterns; refusals.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr};
use crate::physics::flag_result::{FlagResult, flags_entry};
use crate::physics::gear_bit::{GEAR_BITS_RVA, bits_at};

/// The Rust version on the bits at `bits`; `eax` what eax held at entry.
// Converted: `rc2k_core::physics::gearbox::GearBits::lower_free`.
pub unsafe fn lower_free_gear_at(bits: u32, gear: u32, eax: u32) -> Result<FlagResult, Refused> {
    let s = unsafe { bits_at(bits) }.lower_free(gear, eax)?;
    Ok(FlagResult {
        eax: s.bit,
        ecx: Some(s.gear),
        ebx: None,
        carry: !s.found,
        zero: None,
    })
}

/// The Rust version.
pub unsafe fn lower_free_gear(gear: u32, eax: u32) -> Result<FlagResult, Refused> {
    unsafe { lower_free_gear_at(game_addr(GEAR_BITS_RVA) as u32, gear, eax) }
}

fn rust(r: &Regs) -> Result<FlagResult, Refused> {
    unsafe { lower_free_gear(r.ebx, r.eax) }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    flags_entry(r, &rust)
}
