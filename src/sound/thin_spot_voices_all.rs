//! `sound::thin_spot_voices_all` (0x40E97D): `sound::thin_spot_voices` for the twelve
//! sounds of which only one voice is heard: 0x1B1 .. 0x1B6, 0x1B9, 0x1BA, 0x1B0, 0x1BE,
//! 0x1C0, 0x1C3.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` / `popa`)
//! Each call with eax the sound and the other registers the one before left. One caller,
//! `sound::spot_sounds`.
//! Shadow mode: both (nothing sent), the voices and variables compared.
//! Self-test: a copy whose calls go to a stand-in, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::sound::turn_parts::turn_entry;

// ---- Rust version ----

pub fn thin_spot_voices_all_in(r: &mut Regs) -> Result<(), Refused> {
    // Converted: `rc2k_core::sound::spot_voices::THINNED_SOUNDS`.
    let mut c = *r;
    for sound in rc2k_core::sound::spot_voices::THINNED_SOUNDS {
        c.eax = sound;
        crate::sound::thin_spot_voices::thin_spot_voices_in(&mut c)?;
    }
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    turn_entry(r, thin_spot_voices_all_in)
}
