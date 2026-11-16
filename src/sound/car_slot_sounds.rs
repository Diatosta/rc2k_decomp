//! `sound::car_slot_sounds`: a player's car's timed sounds (the slots of `sound::play_timed_slot`
//! in its object).
//!
//! Takes the car's sound object and the request. Nothing for a computer's car (negative
//! `sound_offset`). The car's timed slots are played at the car's place (the fourth only when
//! `view_flag` is 0), then `slot_b` at `place_b`. Refuses (before anything) a null object; what the
//! slots refuse puts back what was written (sounds already sent stay sent).

use crate::errors::Refused;
use crate::sound::sets_store::SoundRequest;
use rc2k_core::sound::engine::{computers, placed_slots};

/// The timed slots of object `so`, through `request`.
pub fn car_slot_sounds(
    so: &crate::sound::store::SoundObject,
    request: &SoundRequest,
    sound: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
) -> Result<(), Refused> {
    // The logic is `rc2k_core::sound::engine` (`computers`, `placed_slots`).
    if computers(so.sound_offset.get()) {
        return Ok(());
    }
    let place = |from: &[crate::memory::session::PlainFloat; 3]| {
        for (cell, f) in request.place.iter().zip(from) {
            cell.set(f.get());
        }
    };
    place(&so.place);
    for at in placed_slots(so.view_flag.get() == 0) {
        crate::sound::play_timed_slot::play_timed_slot(so.slot_at(at), request, sound, audio)?;
    }
    place(&so.place_b);
    crate::sound::play_timed_slot::play_timed_slot(&so.slot_b, request, sound, audio)
}
