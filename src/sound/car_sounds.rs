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
//! Shadow mode runs the original alone (it sends sounds).
//! Self-test: a copy whose nine calls go to stand-ins, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, rva};
use crate::sound::turn_parts::{OBJECTS, REQUEST, turn_entry};

// ---- Rust version ----

pub fn car_sounds_in(r: &mut Regs) -> Result<(), Refused> {
    if r.eax > 7 {
        return Err(BadInput("a car's sound object past 7").into());
    }
    let object: u32 = unsafe { game_read(rva(OBJECTS + 4 * r.eax as usize)) };
    if object == 0 {
        return Err(BadInput("a null sound object").into());
    }
    {
        let mut c = Regs {
            ebx: object,
            esi: game_addr(rva(REQUEST)) as u32,
            ..*r
        };
        crate::sound::car_engine_state::car_engine_state_in(&mut c)?;
        unsafe {
            let (o, q) = (c.ebx as *const u32, c.esi as *mut u32);
            let pitch = o.add(0x13C / 4).read_unaligned();
            q.add(8).write_unaligned(pitch);
            c.eax = o.add(0x140 / 4).read_unaligned();
            q.add(9).write_unaligned(c.eax);
        }
        use crate::sound::*;
        revs_sound::revs_sound_in(&mut c)?;
        throttle_sound::throttle_sound_in(&mut c)?;
        speed_sound::speed_sound_in(&mut c)?;
        surface_sounds::surface_sounds_in(&mut c)?;
        car_slot_sounds::car_slot_sounds_in(&mut c)?;
        car_hit_sound::car_hit_sound_in(&mut c)?;
        car_cue_sound::car_cue_sound_in(&mut c)?;
        car_gear_sounds::car_gear_sounds_in(&mut c)
    }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    turn_entry(r, car_sounds_in)
}
