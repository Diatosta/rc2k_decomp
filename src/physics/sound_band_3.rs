//! `physics::sound_band_3` (0x45D0F1): `flt_6D3CD0` x `flt_6D1AE8`, less 10, held to 0 ..= 0x40, in ebx.
//! See `physics/sound_bands.rs` (the core and the test).
//!
//! In:  nothing
//! Out: ebx the band; the rest kept
//!
//! Caller: `sub_45CE32`.
//! Self-test: `sound_bands::test_twin`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::sound_bands::{Twin, band_plain};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

pub const TWIN: Twin = (0x45D0F1, 0x6D3CD0, 0x6D1AE8, 10);

/// The converted version (plain floats).
// Converted: `rc2k_core::physics::sound_bands::sound_band`.
pub unsafe fn sound_band_3_plain() -> Left {
    unsafe { band_plain(TWIN) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { sound_band_3_plain() })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
