//! `sound::car_gear_sounds`: a car's three timed sounds placed behind it: `slots[5]` (armed by
//! `sound::car_engine_state` when the throttle drops at high revs), `slots[6]` (armed by its
//! countdown at low throttle) and `gear_slot` (armed with a gear change; its pitch is
//! `gear_pitch`).
//!
//! Takes the car's sound object and the request. Nothing in state 3. The request's place is the
//! car's place less its motion; the first two slots are played (`sound::play_timed_slot`); then the
//! request's pitch factor is multiplied by `gear_pitch` for the gear slot and put back after.
//! Refuses (before anything) a null object; what the slots refuse puts back what was written
//! (sounds already sent stay sent).

use crate::errors::Refused;
use crate::sound::sets_store::SoundRequest;
use crate::sound::store::{GEAR_SLOT_AT, ObjectSlot, SLOT_FIVE_AT, SLOT_SIX_AT};

/// The gear sounds of object `so`, through `request`.
pub(super) fn car_gear_sounds(
    so: &crate::sound::store::SoundObjectCell,
    request: &mut SoundRequest,
    sound: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
) -> Result<(), Refused> {
    if so.borrow().state == 3 {
        return Ok(());
    }
    // The logic is `rc2k_core::sound::engine::{place_behind, gear_pitch}` (exact at every
    // precision: one rounding each).
    use rc2k_core::sound::engine::{gear_pitch, place_behind};
    request.place = {
        let o = so.borrow();
        place_behind(o.place, o.motion)
    };
    for at in [SLOT_FIVE_AT, SLOT_SIX_AT] {
        let slot = ObjectSlot::new(so, at);
        crate::sound::play_timed_slot::play_timed_slot(&slot, request, sound, audio)?;
    }
    let kept = request.pitch_factor;
    request.pitch_factor = gear_pitch(kept, so.borrow().gear_pitch);
    let slot = ObjectSlot::new(so, GEAR_SLOT_AT);
    crate::sound::play_timed_slot::play_timed_slot(&slot, request, sound, audio)?;
    request.pitch_factor = kept;
    Ok(())
}
