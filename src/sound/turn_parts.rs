//! The race sounds' turn: `sound::race_turn` and the trees under it, which work on the eight cars'
//! sound objects, the shared spots' voices (`SpotVoices.voices[0][0]`) and one request
//! (`SoundRequest`, ten dwords, see `sound::play_placed_sound`) they send each sound with.
//! This file: what they share, and the three ways a sound goes to the audio thread.
//!
//! Every function of the turn takes what it reads as arguments. The words a sound command carries
//! besides those `play_sound` sets are unused; the audio thread reads none of them (a stop is the
//! sound and a volume of 0: `rc2k_core::audio::voices::plan`), so they are 0.

use crate::audio::owner::Audio;
use crate::errors::Refused;
use crate::sound::owner::Sound;
use crate::sound::sets_store::SoundRequest;

/// The request: the sound command's words.
pub fn request(sound: &Sound) -> &SoundRequest {
    sound.race.request.get()
}

pub use crate::sound::play_sound::play_sound_in as play;

/// `sound::stop_sound`: the request's sound stopped unless it has bit 0x8000.
pub fn stop(request: &SoundRequest, audio: &Audio) -> Result<(), Refused> {
    crate::sound::stop_sound::stop_sound(request, audio);
    Ok(())
}

/// `sound::play_placed_sound` with the request.
pub fn placed(request: &SoundRequest, sound: &Sound, audio: &Audio) -> Result<(), Refused> {
    Ok(crate::sound::play_placed_sound::play_placed_sound_in_game(
        request, sound, audio,
    )?)
}

/// `sound::step_race_clock`: true when a turn is due.
pub fn step_clock(sound: &Sound, system: &crate::system::owner::System) -> bool {
    crate::sound::race_clock::step_race_clock(
        crate::system::tick_count::tick_count(&system.session.picture_clock),
        sound,
    )
}
