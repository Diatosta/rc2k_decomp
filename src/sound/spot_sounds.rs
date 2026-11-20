//! `sound::spot_sounds`: the shared spots' sounds for the turn.
//!
//! Takes the request (the sound request being worked on). Runs `sound::match_spot_voices`,
//! `free_spot_voices`, `spot_voice_distances`, `merge_spot_voices_all`, `thin_spot_voices_all`,
//! `exclusive_spot_voices`, `repeat_spot_voices` and `play_spot_voices` in that order. See
//! `sound/spot_voices.rs`. What the callees refuse puts back what was written (sounds already sent
//! stay sent).

use crate::errors::Refused;

/// The shared spots' voices for the turn.
pub(super) fn spot_sounds(
    sound: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
    system: &crate::system::owner::System,
    random: &crate::math::next_random::Randoms,
) -> Result<(), Refused> {
    let mut request = crate::sound::turn_parts::request(sound);
    use crate::sound::*;
    match_spot_voices::match_spot_voices(sound)?;
    free_spot_voices::free_spot_voices(&mut request, sound, audio)?;
    spot_voice_distances::spot_voice_distances(sound);
    merge_spot_voices_all::merge_spot_voices_all(sound, system)?;
    thin_spot_voices_all::thin_spot_voices_all(sound, system, random)?;
    exclusive_spot_voices::exclusive_spot_voices(sound);
    repeat_spot_voices::repeat_spot_voices(sound, random)?;
    play_spot_voices::play_spot_voices(&mut request, audio, sound)
}
