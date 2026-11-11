//! `sound::car_slot_sounds` (0x40E0F6): a player's car's timed sounds (the slots of
//! `sound::play_timed_slot` in its object).
//!
//! Takes: the car's sound object, the request
//! Nothing for a computer's car (+0 negative). The request's place = +0x2C .. +0x34;
//! the slots at +0x164, +0x14C, +0x17C, +0x194 (only with +0xC 0), +0x260 and +0x1AC;
//! then the place = +0x214 .. +0x21C and the slot at +0x220. One caller, `sound::car_sounds`.
//! Refuses (before anything) a null object; what the slots refuse puts back
//! what was written (sounds already sent stay sent).

use crate::errors::Refused;
use crate::sound::sets_store::SoundRequest;
use rc2k_core::sound::engine::{computers, placed_slots};

/// The timed slots of object `so`, through `request`.
pub fn car_slot_sounds(
    so: &crate::sound::store::SoundObject,
    request: &SoundRequest,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::sound::engine` (`computers`, `placed_slots`).
    if computers(so.sound_offset.get()) {
        return Ok(());
    }
    let place = |from: &[crate::session::Float; 3]| {
        for (cell, f) in request.place.iter().zip(from) {
            cell.set(f.bits());
        }
    };
    place(&so.place);
    for at in placed_slots(so.view_flag.get() == 0) {
        crate::sound::play_timed_slot::play_timed_slot(so.slot_at(at), request)?;
    }
    place(&so.place_b);
    crate::sound::play_timed_slot::play_timed_slot(&so.slot_b, request)
}
