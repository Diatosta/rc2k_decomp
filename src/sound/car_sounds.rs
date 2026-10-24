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
use crate::sound::turn_parts::OBJECTS;

// ---- Rust version ----

/// Car `n`'s sounds for the turn.
pub fn car_sounds(n: u32) -> Result<(), Refused> {
    if n > 7 {
        return Err(BadInput("a car's sound object past 7").into());
    }
    let object: u32 = unsafe { crate::store::game_read(OBJECTS + 4 * n as usize) };
    if object == 0 {
        return Err(BadInput("a null sound object").into());
    }
    let request = crate::sound::turn_parts::request();
    let kind = crate::sound::car_engine_state::car_engine_state(object)?;
    unsafe {
        let (o, q) = (object as *const u32, request as *mut u32);
        q.add(8).write_unaligned(o.add(0x13C / 4).read_unaligned());
        q.add(9).write_unaligned(o.add(0x140 / 4).read_unaligned());
    }
    use crate::sound::*;
    revs_sound::revs_sound(object, kind, request)?;
    throttle_sound::throttle_sound(object, kind, request)?;
    speed_sound::speed_sound(object, kind, request)?;
    surface_sounds::surface_sounds(object, request)?;
    car_slot_sounds::car_slot_sounds(object, request)?;
    car_hit_sound::car_hit_sound(object, request)?;
    car_cue_sound::car_cue_sound(object, request)?;
    car_gear_sounds::car_gear_sounds(object, request)
}
