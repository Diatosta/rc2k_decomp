//! `sound::spot_sounds` (0x40E611): the shared spots' sounds for the turn.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` / `popa`)
//! esi the request; then `sound::match_spot_voices`, `free_spot_voices`,
//! `spot_voice_distances`, `merge_spot_voices_all`, `thin_spot_voices_all`,
//! `exclusive_spot_voices`, `repeat_spot_voices`, `play_spot_voices`, each getting the
//! registers the one before left. See `sound/spot_voices.rs`. One caller,
//! `sound::race_turn`.
//! What the callees refuse puts back what was written (sounds already sent stay sent).
//! Shadow mode runs the original alone (it sends sounds).
//! Self-test: a copy whose eight calls go to stand-ins, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};
use crate::sound::turn_parts::{Part, REQUEST, turn_entry};

const STEPS: [u32; 8] = [
    0x40E642, 0x40E6F4, 0x40E733, 0x40E7CA, 0x40E97D, 0x40EAF3, 0x40EB44, 0x40EB9E,
];

// ---- Rust version ----

pub fn spot_sounds_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    let mut c = Regs {
        esi: game_addr(rva(REQUEST)) as u32,
        ..*r
    };
    for step in STEPS {
        part(step, &mut c)?;
    }
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    turn_entry(r, spot_sounds_in)
}
