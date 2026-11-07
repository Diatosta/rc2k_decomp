//! The race sounds' turn (stream B, 2026-10-18): `sound::race_turn` (0x40D5F3) and the
//! trees under it, which work on the eight cars' sound objects (`g_raceSoundObjects`,
//! 0x4A8 bytes each), the shared spots' voices (`unk_5168C0`) and one request
//! (`unk_5124BC`, ten dwords, see `sound::play_placed_sound`) they send each sound with.
//! This file: what they share, and the three ways a sound goes to the audio thread.
//!
//! Plain functions since 2026-10-23 (phase 3): every function of the turn keeps the
//! registers (`pusha` / `popa`, or none changed that a caller reads), so they take what
//! they read as arguments. The registers a sound command is queued with besides those
//! `play_sound` sets were the callers' leftovers; the audio thread reads none of them
//! (a stop is the sound and a volume of 0: `rc2k_core::audio::voices::plan`), so they
//! are 0 now.

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
