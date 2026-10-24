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
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;

// ---- Rust version ----

/// The shared spots' voices for the turn.
pub fn spot_sounds() -> Result<(), Refused> {
    let request = crate::sound::turn_parts::request();
    use crate::sound::*;
    match_spot_voices::match_spot_voices()?;
    free_spot_voices::free_spot_voices(request)?;
    spot_voice_distances::spot_voice_distances();
    merge_spot_voices_all::merge_spot_voices_all()?;
    thin_spot_voices_all::thin_spot_voices_all()?;
    exclusive_spot_voices::exclusive_spot_voices();
    repeat_spot_voices::repeat_spot_voices()?;
    play_spot_voices::play_spot_voices(request)
}
