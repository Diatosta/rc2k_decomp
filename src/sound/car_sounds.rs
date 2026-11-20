//! `sound::car_sounds`: one car's sounds for the turn.
//!
//! Takes the car's number; the request is `SoundRequest`. Runs `sound::car_engine_state`
//! on the object, sets the request's pitch and volume factors from the object's, then plays
//! `sound::revs_sound`, `sound::throttle_sound`, `sound::speed_sound`, `sound::surface_sounds`,
//! `sound::car_slot_sounds`, `sound::car_hit_sound`, `sound::car_cue_sound` and
//! `sound::car_gear_sounds` (the engine state's result, the car's sound kind, goes to the next
//! three). Three calls, from `sound::car_sounds_all`. Refuses (before anything) a number over 7 and
//! a null object; what the callees refuse puts back what was written (sounds already sent stay
//! sent).

use crate::errors::{BadInput, Refused};

/// Car `n`'s sounds for the turn.
pub fn car_sounds(
    n: u32,
    sound: &crate::sound::owner::Sound,
    race: &crate::race::owner::Race,
    system: &crate::system::owner::System,
    audio: &crate::audio::owner::Audio,
) -> Result<(), Refused> {
    if n > 7 {
        return Err(BadInput("a car's sound object past 7").into());
    }
    let so = crate::sound::store::SoundObject::of(sound, n)?;
    let mut request = crate::sound::turn_parts::request(sound);
    let kind = crate::sound::car_engine_state::car_engine_state(so, sound, race, system)?;
    request.pitch_factor = so.pitch_factor.get();
    request.volume_factor = so.volume_factor.get();
    use crate::sound::*;
    revs_sound::revs_sound(so, kind, &mut request, sound, audio)?;
    throttle_sound::throttle_sound(so, kind, &mut request, sound, audio)?;
    speed_sound::speed_sound(so, kind, &mut request, sound, audio)?;
    surface_sounds::surface_sounds(so, &mut request, sound, system, audio)?;
    car_slot_sounds::car_slot_sounds(so, &mut request, sound, audio)?;
    car_hit_sound::car_hit_sound(so, &mut request, sound, audio)?;
    car_cue_sound::car_cue_sound(so, &mut request, sound, audio)?;
    car_gear_sounds::car_gear_sounds(so, &mut request, sound, audio)
}
