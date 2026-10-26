//! `sound::car_gear_sounds` (0x40E24F): a car's three timed sounds placed behind it: the
//! slots at +0x1C4 (armed by `sound::car_engine_state` when the throttle drops at high
//! revs), +0x1DC (armed by its countdown at low throttle) and +0x1F8 (armed with a gear
//! change; its pitch +0x210). Names by those readings.
//!
//! In:  ebx the car's sound object, esi the request
//! Out: every register preserved (`pusha` / `popa`)
//! Nothing in state 3 (+0x10C). The request's place = +0x2C .. +0x34 less +0x38 ..
//! +0x40; the slots at +0x1C4 and +0x1DC (`sound::play_timed_slot`); then, the
//! request's pitch factor (+0x20) times the object's +0x210 for it and put back after,
//! the slot at +0x1F8. One caller, `sound::car_sounds`.
//! Refuses (before anything) a null object or request; what the slots refuse puts back
//! what was written (sounds already sent stay sent).
//! Shadow mode runs the original alone (it sends sounds).
//! Self-test: a copy whose calls go to a stand-in, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::Regs;
use crate::physics::flow::guard;
use crate::sound::car_slot_sounds::SLOT;
use crate::sound::turn_parts::{OBJECT_LEN, Part, turn_entry, turn_regions};

// ---- Rust version ----

pub fn car_gear_sounds_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    if r.ebx == 0 || r.esi == 0 {
        return Err(BadInput("no object or no request").into());
    }
    let o = r.ebx;
    if unsafe { ((o + 0x10C) as *const u32).read_unaligned() } == 3 {
        return Ok(());
    }
    let mut all = turn_regions();
    all.push((o, OBJECT_LEN));
    guard(&all, || {
        // Converted: `rc2k_core::sound::engine::{place_behind, gear_pitch}` (exact at every
        // precision: one rounding each).
        use rc2k_core::sound::engine::{gear_pitch, place_behind};
        let mut c = *r;
        let float = |at: u32| unsafe { (at as *const f32).read_unaligned() };
        let place = place_behind(
            [0x2C, 0x30, 0x34].map(|n| float(c.ebx + n)),
            [0x38, 0x3C, 0x40].map(|n| float(c.ebx + n)),
        );
        for (n, v) in place.into_iter().enumerate() {
            unsafe { ((c.esi + 0xC + 4 * n as u32) as *mut f32).write_unaligned(v) };
        }
        for at in [0x1C4, 0x1DC] {
            c.ebp = o + at;
            part(SLOT, &mut c)?;
        }
        let factor = (c.esi + 0x20) as *mut u32;
        let kept = unsafe { factor.read_unaligned() };
        let pitch = gear_pitch(float(c.esi + 0x20), float(c.ebx + 0x210));
        unsafe { (factor as *mut f32).write_unaligned(pitch) };
        c.ebp = o + 0x1F8;
        part(SLOT, &mut c)?;
        unsafe { factor.write_unaligned(kept) };
        Ok(())
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    turn_entry(r, car_gear_sounds_in)
}
