//! `sound::car_gear_sounds` (0x40E24F): a car's three timed sounds placed behind it: the
//! slots at +0x1C4 (armed by `sound::car_engine_state` when the throttle drops at high
//! revs), +0x1DC (armed by its countdown at low throttle) and +0x1F8 (armed with a gear
//! change; its pitch +0x210). Names by those readings.
//!
//! Takes: the car's sound object, the request
//! Nothing in state 3 (+0x10C). The request's place = +0x2C .. +0x34 less +0x38 ..
//! +0x40; the slots at +0x1C4 and +0x1DC (`sound::play_timed_slot`); then, the
//! request's pitch factor (+0x20) times the object's +0x210 for it and put back after,
//! the slot at +0x1F8. One caller, `sound::car_sounds`.
//! Refuses (before anything) a null object; what the slots refuse puts back
//! what was written (sounds already sent stay sent).

use crate::errors::Refused;
use crate::sound::sets_store::SoundRequest;

/// The gear sounds of object `so`, through `request`.
pub fn car_gear_sounds(
    so: &crate::sound::store::SoundObject,
    request: &SoundRequest,
) -> Result<(), Refused> {
    if so.state.get() == 3 {
        return Ok(());
    }
    // Converted: `rc2k_core::sound::engine::{place_behind, gear_pitch}` (exact at every
    // precision: one rounding each).
    use rc2k_core::sound::engine::{gear_pitch, place_behind};
    let place = place_behind(
        so.place.each_ref().map(super::super::session::Float::get),
        so.motion.each_ref().map(super::super::session::Float::get),
    );
    for (cell, v) in request.place.iter().zip(place) {
        cell.set(v.to_bits());
    }
    for slot in &so.slots[5..] {
        crate::sound::play_timed_slot::play_timed_slot(slot, request)?;
    }
    let factor = &request.pitch_factor;
    let kept = factor.get();
    let pitch = gear_pitch(kept, so.gear_pitch.get());
    factor.set(pitch);
    crate::sound::play_timed_slot::play_timed_slot(&so.gear_slot, request)?;
    factor.set(kept);
    Ok(())
}
