//! `sound::car_slot_sounds`: a player's car's timed sounds (the slots of `sound::play_timed_slot`
//! in its object).
//!
//! Takes the car's sound object and the request. Nothing for a computer's car (negative
//! `sound_offset`). The car's timed slots are played at the car's place (the fourth only when
//! `view_flag` is 0), then `slot_b` at `place_b`. Refuses (before anything) a null object; what the
//! slots refuse puts back what was written (sounds already sent stay sent).

use crate::errors::Refused;
use crate::sound::sets_store::SoundRequest;
use crate::sound::store::{ObjectSlot, SLOT_B_AT};
use rc2k_core::sound::engine::{computers, placed_slots};

/// The timed slots of object `so`, through `request`.
pub fn car_slot_sounds(
    so: &crate::sound::store::SoundObjectCell,
    request: &mut SoundRequest,
    sound: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
) -> Result<(), Refused> {
    // The logic is `rc2k_core::sound::engine` (`computers`, `placed_slots`).
    if computers(so.borrow().sound_offset) {
        return Ok(());
    }
    request.place = so.borrow().place;
    let view_flag = so.borrow().view_flag;
    for at in placed_slots(view_flag == 0) {
        let slot = ObjectSlot::new(so, at);
        crate::sound::play_timed_slot::play_timed_slot(&slot, request, sound, audio)?;
    }
    request.place = so.borrow().place_b;
    let slot = ObjectSlot::new(so, SLOT_B_AT);
    crate::sound::play_timed_slot::play_timed_slot(&slot, request, sound, audio)
}
