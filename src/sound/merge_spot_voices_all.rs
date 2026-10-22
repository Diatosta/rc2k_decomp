//! `sound::merge_spot_voices_all` (0x40E7CA): `sound::merge_spot_voices` for the six
//! sounds whose voices are merged into one: 0x1B7, 0x1B8, 0x1BB, 0x1BC, 0x1C1, 0x1BF.
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
use crate::sound::turn_parts::{Part, turn_entry};

/// `sound::merge_spot_voices`.
const EACH: u32 = 0x40E809;

// ---- Rust version ----

pub fn merge_spot_voices_all_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    // Converted: `rc2k_core::sound::spot_voices::MERGED_SOUNDS`.
    let mut c = *r;
    for sound in rc2k_core::sound::spot_voices::MERGED_SOUNDS {
        c.eax = sound;
        part(EACH, &mut c)?;
    }
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    turn_entry(r, merge_spot_voices_all_in)
}
