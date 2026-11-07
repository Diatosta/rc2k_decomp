//! `sound::car_sounds` (0x40D893): one car's sounds for the turn.
//!
//! In:  eax the car's number (`g_raceSoundObjects[eax]`)
//! Out: every register preserved (`pusha` / `popa`)
//! ebx the object, esi the request (`unk_5124BC`); `sound::car_engine_state`, then the
//! request's +0x20 and +0x24 (the pitch and volume factors) = the object's +0x13C and
//! +0x140 (eax left at +0x140), then `sound::revs_sound`, `sound::throttle_sound`,
//! `sound::speed_sound`, `sound::surface_sounds`, `sound::car_slot_sounds`,
//! `sound::car_hit_sound`, `sound::car_cue_sound`, `sound::car_gear_sounds`, each
//! getting the registers the one before left (the engine state leaves edi the car's
//! sound kind, which the next three read). Three calls, from `sound::car_sounds_all`.
//! Refuses (before anything) a number over 7 and a null object; what the callees refuse
//! puts back what was written (sounds already sent stay sent).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};

// ---- Rust version ----

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
