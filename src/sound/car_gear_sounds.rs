//! `sound::car_gear_sounds`: a car's three timed sounds placed behind it: `slots[5]` (armed by
//! `sound::car_engine_state` when the throttle drops at high revs), `slots[6]` (armed by its
//! countdown at low throttle) and `gear_slot` (armed with a gear change; its pitch is
//! `gear_pitch`).
//!
//! Takes the car's sound object and the request. Nothing in state 3. The request's place is the
//! car's place less its motion; the first two slots are played (`sound::play_timed_slot`); then the
//! request's pitch factor is multiplied by `gear_pitch` for the gear slot and put back after. One
//! caller, `sound::car_sounds`. Refuses (before anything) a null object; what the slots refuse puts
//! back what was written (sounds already sent stay sent).

use crate::errors::Refused;
use crate::sound::sets_store::SoundRequest;

/// The gear sounds of object `so`, through `request`.
pub fn car_gear_sounds(
    so: &crate::sound::store::SoundObject,
    request: &SoundRequest,
    sound: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
) -> Result<(), Refused> {
    if so.state.get() == 3 {
        return Ok(());
    }
    // The logic is `rc2k_core::sound::engine::{place_behind, gear_pitch}` (exact at every
    // precision: one rounding each).
    use rc2k_core::sound::engine::{gear_pitch, place_behind};
    let place = place_behind(
        so.place
            .each_ref()
            .map(crate::memory::session::PlainFloat::get),
        so.motion
            .each_ref()
            .map(crate::memory::session::PlainFloat::get),
    );
    for (cell, v) in request.place.iter().zip(place) {
        cell.set(v);
    }
    for slot in &so.slots[5..] {
        crate::sound::play_timed_slot::play_timed_slot(slot, request, sound, audio)?;
    }
    let factor = &request.pitch_factor;
    let kept = factor.get();
    let pitch = gear_pitch(kept, so.gear_pitch.get());
    factor.set(pitch);
    crate::sound::play_timed_slot::play_timed_slot(&so.gear_slot, request, sound, audio)?;
    factor.set(kept);
    Ok(())
}
