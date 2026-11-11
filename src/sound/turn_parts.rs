//! The race sounds' turn: `sound::race_turn` (0x40D5F3) and the
//! trees under it, which work on the eight cars' sound objects (`g_raceSoundObjects`,
//! 0x4A8 bytes each), the shared spots' voices (`SpotVoices.voices[0][0]`) and one request
//! (`RaceSoundSets.request.sound`, ten dwords, see `sound::play_placed_sound`) they send each sound with.
//! This file: what they share, and the three ways a sound goes to the audio thread.
//!
//! Every function of the turn takes what it reads as arguments. The words a sound command
//! carries besides those `play_sound` sets were leftovers in the original; the audio thread
//! reads none of them (a stop is the sound and a volume of 0:
//! `rc2k_core::audio::voices::plan`), so they are 0.

use crate::errors::Refused;
use crate::sound::sets_store::SoundRequest;

/// The request (`g_raceSoundRequest`), the sound command's words.
pub fn request() -> &'static SoundRequest {
    &crate::sound::owner::sound().race.sets.request
}

pub use crate::sound::play_sound::play_sound as play;

/// `sound::stop_sound`: the request's sound stopped unless it has bit 0x8000.
pub fn stop(request: &SoundRequest) -> Result<(), Refused> {
    crate::sound::stop_sound::stop_sound(request);
    Ok(())
}

/// `sound::play_placed_sound` with the request.
pub fn placed(request: &SoundRequest) -> Result<(), Refused> {
    let audio = crate::audio::owner::audio();
    Ok(crate::sound::play_placed_sound::play_placed_sound_in_game(
        request, audio,
    )?)
}

/// `sound::step_race_clock`: true when a turn is due (the original's carry clear).
pub fn step_clock(sound: &crate::sound::owner::Sound) -> bool {
    crate::sound::race_clock::step_race_clock(crate::system::tick_count::tick_count(), sound)
}
