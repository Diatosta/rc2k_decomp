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
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;

// ---- Rust version ----

/// Converted: `rc2k_core::sound::spot_voices::thin` and `thin_choose`. The voices of
/// `sound` thinned.
pub fn thin_spot_voices(sound: u32) -> Result<(), Refused> {
    use crate::sound::spot_voices::{load_voices, store_picks, store_voices};
    use rc2k_core::sound::spot_voices as core;
    let mut voices = load_voices();
    let (picks, trail) = core::thin(&mut voices, sound);
    store_voices(&voices);
    store_picks(&picks, &trail);
    if trail.ended_at.is_some() {
        return Ok(());
    }
    let (Some(_), Some(_)) = (picks.nearest, picks.next) else {
        return Ok(());
    };
    let random = unsafe { crate::math::next_random::next_random() };
    core::thin_choose(&mut voices, &picks, random);
    store_voices(&voices);
    Ok(())
}
