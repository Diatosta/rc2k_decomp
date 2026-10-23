//! `physics::rate_current_car` (0x4AB5D0): `call physics::rate_damage; retn`.
//!
//! In:  nothing
//! Out: as `rate_damage` (eax the gear bits, ecx 9, esi 0x75B568)
//!
//! Caller: `physics::grade_or_reset` (0x43BA9E, `dword_71A234` not 2).
//! Self-test: a copy of the original (its call to a copy of `rate_damage` on the
//! mirror) against the Rust core, as `rate_damage`'s.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::mirror::Spec;
use crate::physics::rate_damage::rate_damage;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

pub const OWN_SPEC: Spec = (0x4AB5D0, 6, &[0xE8, 0x17, 0x3C, 0x00, 0x00], &[0], 0);

/// The Rust version.
pub unsafe fn rate_current_car() -> Left {
    unsafe { rate_damage() }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { rate_current_car() })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
