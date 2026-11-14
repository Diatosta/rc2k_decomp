//! `sound::stop_sound`: plays a request's sound at volume 0.
//!
//! Unless the sound has bit 0x8000: `sound::play_sound` with the sound and volume 0. That it stops
//! the sound is a reading (a volume of 0). 14 callers (the race's sounds).

use crate::sound::sets_store::SoundRequest;

/// The request's sound stopped (`sound::play_sound` with a volume of 0, of which the
/// audio thread reads only the sound and the volume) unless it has bit 0x8000.
pub fn stop_sound(request: &SoundRequest, audio: &crate::audio::owner::Audio) {
    let sound = request.sound.get();
    // The logic is `rc2k_core::sound::senders::stopped`.
    if let Some(sound) = rc2k_core::sound::senders::stopped(sound) {
        crate::sound::play_sound::play_sound_in(audio, sound, 0, 0, [0; 3]);
    }
}
