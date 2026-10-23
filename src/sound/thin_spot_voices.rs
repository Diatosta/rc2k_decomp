//! `sound::thin_spot_voices` (0x40E9F8): of the voices heard with one sound, the nearest
//! two found and one of them, by a random bit, kept heard; unless one is playing.
//!
//! In:  eax the sound
//! Out: ebp the sound; the others as the original leaves them (see the code); esi, edi
//!      preserved
//! The pickers' work cleared. For each voice heard with the sound: one already playing
//! ends it at once (nothing more changes). Else its distance x 1000 (`fistp`) against
//! the nearest's and the next's (unsigned): a new nearest moves the old one to next and
//! stays heard; a new next, and every other, is silenced (+4 0). A nearest displaced
//! stays heard (the code that would silence it, 0x40EA9B, is never reached: notes'
//! bugs). With a next found: `math::next_random` & 1 for the nearest's +4, the other bit
//! for the next's. Twelve callers, `sound::thin_spot_voices_all`.
//! Shadow mode: both (nothing sent; the random state put back between), compared.
//! Self-test: a copy whose call goes to a stand-in (the test's random numbers), against
//! the Rust version, on the game's voices (put back).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::sound::spot_voices::voice;
use crate::sound::turn_parts::{Part, RANDOM, turn_entry};

// ---- Rust version ----

/// Converted: `rc2k_core::sound::spot_voices::thin` and `thin_choose`.
pub fn thin_spot_voices_in(c: &mut Regs, part: Part) -> Result<(), Refused> {
    use crate::sound::spot_voices::{load_voices, pick_regs, store_picks, store_voices};
    use rc2k_core::sound::spot_voices as core;
    let mut voices = load_voices();
    let (picks, trail) = core::thin(&mut voices, c.eax);
    store_voices(&voices);
    store_picks(&picks, &trail);
    *c = pick_regs(c, &trail);
    if trail.ended_at.is_some() {
        return Ok(());
    }
    let (Some(nearest), Some(next)) = (picks.nearest, picks.next) else {
        return Ok(());
    };
    (c.ebx, c.edx) = (voice(nearest as u32), voice(next as u32));
    part(RANDOM, c)?;
    core::thin_choose(&mut voices, &picks, c.eax);
    store_voices(&voices);
    c.eax = (c.eax & 1) ^ 1;
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    turn_entry(r, thin_spot_voices_in)
}
