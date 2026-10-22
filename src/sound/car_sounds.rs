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
use crate::physics::flow::guard;
use crate::sound::turn_parts::{OBJECTS, Part, REQUEST, turn_entry, turn_regions};

const ENGINE: u32 = 0x40D8E2;
/// After the factors, in the original's order.
const SOUNDS: [u32; 8] = [
    0x40DD19, 0x40DD96, 0x40DE09, 0x40DE9B, 0x40E0F6, 0x40E182, 0x40E1E5, 0x40E24F,
];

// ---- Rust version ----

pub fn car_sounds_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    if r.eax > 7 {
        return Err(BadInput("a car's sound object past 7").into());
    }
    let object: u32 = unsafe { game_read(rva(OBJECTS + 4 * r.eax as usize)) };
    if object == 0 {
        return Err(BadInput("a null sound object").into());
    }
    guard(&turn_regions(), || {
        let mut c = Regs {
            ebx: object,
            esi: game_addr(rva(REQUEST)) as u32,
            ..*r
        };
        part(ENGINE, &mut c)?;
        unsafe {
            let (o, q) = (c.ebx as *const u32, c.esi as *mut u32);
            let pitch = o.add(0x13C / 4).read_unaligned();
            q.add(8).write_unaligned(pitch);
            c.eax = o.add(0x140 / 4).read_unaligned();
            q.add(9).write_unaligned(c.eax);
        }
        for place in SOUNDS {
            part(place, &mut c)?;
        }
        Ok(())
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    turn_entry(r, car_sounds_in)
}
