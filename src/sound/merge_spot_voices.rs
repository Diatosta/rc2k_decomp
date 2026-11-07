//! `sound::merge_spot_voices` (0x40E809): of the voices heard with one sound, the
//! nearest kept and (re)started, moved to the point between it and the next nearest
//! that is nearest the listener; all the others silenced.
//!
//! In:  eax the sound
//! Out: ebp the sound, ecx 0; ebx the nearest (or the end of the voices), eax and edx
//!      as the original leaves them (see the code); esi, edi preserved
//! The pickers' work (`dword_517F40` .. `dword_517F50`) cleared (distances to -1). For
//! each voice heard with the sound: a playing one noted in `dword_517F40`; its distance
//! x 1000 as a whole number (`fistp` through `g_soundScratch`) compared (unsigned) with
//! the nearest's and the next's, which move down; the voice silenced (+4 and +0x28 0).
//! The nearest, if any, gets +0x24 and +4 1; with a next as well, `math::nearest_on_segment`
//! of their places and the listener (0, 0, 0) gives the nearest's new place.
//! Six callers, `sound::merge_spot_voices_all`.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
// ---- Rust version ----

/// Converted: `rc2k_core::sound::spot_voices::merge` and `merge_between`. The voices of
/// `sound` merged.
pub fn merge_spot_voices(sound: u32, owner: &crate::sound::owner::Sound) -> Result<(), Refused> {
    use crate::sound::spot_voices::{load_voices, store_picks, store_voices};
    use rc2k_core::sound::spot_voices as core;
    let mut voices = load_voices(owner);
    let (picks, trail) = core::merge(&mut voices, sound);
    store_voices(owner, &voices);
    store_picks(owner, &picks, &trail);
    let (Some(nearest), Some(next)) = (picks.nearest, picks.next) else {
        return Ok(());
    };
    let (a, b) = (voices[nearest].place, voices[next].place);
    // The ends and the listener (0, 0, 0) through `dword_71C384` .., the point back from
    // `dword_71C3A8` .. in the original: now `nearest_on_segment`'s own.
    let point = crate::math::nearest_on_segment::nearest_on_segment(a, b, [0; 3]);
    core::merge_between(&mut voices, &picks, point);
    store_voices(owner, &voices);
    Ok(())
}
