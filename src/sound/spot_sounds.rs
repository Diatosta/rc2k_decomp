//! `sound::spot_sounds` (0x40E611): the shared spots' sounds for the turn.
//!
//! Takes the request (the sound request being worked on). Runs `sound::match_spot_voices`,
//! `free_spot_voices`, `spot_voice_distances`, `merge_spot_voices_all`,
//! `thin_spot_voices_all`, `exclusive_spot_voices`, `repeat_spot_voices` and
//! `play_spot_voices` in that order. See `sound/spot_voices.rs`. One caller,
//! `sound::race_turn`.
//! What the callees refuse puts back what was written (sounds already sent stay sent).

use crate::errors::Refused;

/// The shared spots' voices for the turn.
pub fn spot_sounds(
    sound: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
) -> Result<(), Refused> {
    let request = crate::sound::turn_parts::request();
    use crate::sound::*;
    match_spot_voices::match_spot_voices(sound)?;
    free_spot_voices::free_spot_voices(request, sound)?;
    spot_voice_distances::spot_voice_distances(sound)?;
    merge_spot_voices_all::merge_spot_voices_all(sound)?;
    thin_spot_voices_all::thin_spot_voices_all(sound)?;
    exclusive_spot_voices::exclusive_spot_voices(sound);
    repeat_spot_voices::repeat_spot_voices(sound)?;
    play_spot_voices::play_spot_voices(request, audio, sound)
}
