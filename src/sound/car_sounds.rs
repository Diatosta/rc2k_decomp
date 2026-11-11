//! `sound::car_sounds` (0x40D893): one car's sounds for the turn.
//!
//! Takes the car's number (`g_raceSoundObjects[]`); the request is `unk_5124BC`.
//! Runs `sound::car_engine_state` on the object, sets the request's +0x20 and +0x24 (the
//! pitch and volume factors) to the object's +0x13C and +0x140, then `sound::revs_sound`, `sound::throttle_sound`,
//! `sound::speed_sound`, `sound::surface_sounds`, `sound::car_slot_sounds`,
//! `sound::car_hit_sound`, `sound::car_cue_sound`, `sound::car_gear_sounds`
//! (the engine state's result, the car's sound kind, goes to the next three). Three calls, from `sound::car_sounds_all`.
//! Refuses (before anything) a number over 7 and a null object; what the callees refuse
//! puts back what was written (sounds already sent stay sent).

use crate::errors::{BadInput, Refused};

/// Car `n`'s sounds for the turn.
pub fn car_sounds(n: u32, sound: &crate::sound::owner::Sound) -> Result<(), Refused> {
    if n > 7 {
        return Err(BadInput("a car's sound object past 7").into());
    }
    let so = crate::sound::store::SoundObject::number(n)?;
    let request = crate::sound::turn_parts::request();
    let kind = crate::sound::car_engine_state::car_engine_state(so, sound)?;
    request.pitch_factor.set(so.pitch_factor.bits());
    request.volume_factor.set(so.volume_factor.bits());
    use crate::sound::*;
    revs_sound::revs_sound(so, kind, request, sound)?;
    throttle_sound::throttle_sound(so, kind, request, sound)?;
    speed_sound::speed_sound(so, kind, request, sound)?;
    surface_sounds::surface_sounds(so, request, sound)?;
    car_slot_sounds::car_slot_sounds(so, request)?;
    car_hit_sound::car_hit_sound(so, request)?;
    car_cue_sound::car_cue_sound(so, request)?;
    car_gear_sounds::car_gear_sounds(so, request)
}
