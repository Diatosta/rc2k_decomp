//! `sound::car_slot_sounds` (0x40E0F6): a player's car's timed sounds (the slots of
//! `sound::play_timed_slot` in its object).
//!
//! In:  ebx the car's sound object, esi the request
//! Out: every register preserved (`pusha` / `popa`)
//! Nothing for a computer's car (+0 negative). The request's place = +0x2C .. +0x34;
//! the slots at +0x164, +0x14C, +0x17C, +0x194 (only with +0xC 0), +0x260 and +0x1AC;
//! then the place = +0x214 .. +0x21C and the slot at +0x220. Each call gets the
//! registers the one before left. One caller, `sound::car_sounds`.
//! Refuses (before anything) a null object or request; what the slots refuse puts back
//! what was written (sounds already sent stay sent).
//! A near `jl` in its first five bytes: private copy (0x8C bytes, the seven calls
//! relocated), manual patch.
//! Shadow mode runs the original alone (it sends sounds).
//! Self-test: a copy whose calls go to a stand-in, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::Regs;
use crate::sound::turn_parts::turn_entry;
use rc2k_core::sound::engine::{computers, placed_slots};

// ---- Rust version ----

pub fn car_slot_sounds_in(r: &mut Regs) -> Result<(), Refused> {
    if r.ebx == 0 || r.esi == 0 {
        return Err(BadInput("no object or no request").into());
    }
    let o = r.ebx;
    let w = |at: u32| unsafe { ((o + at) as *const u32).read_unaligned() };
    // Converted: `rc2k_core::sound::engine` (`computers`, `placed_slots`).
    if computers(w(0)) {
        return Ok(());
    }
    let q = r.esi as *mut u32;
    {
        let mut c = *r;
        let place = |c: &mut Regs, from: u32| {
            for k in 0..3 {
                c.eax = w(from + 4 * k);
                unsafe { q.add(3 + k as usize).write_unaligned(c.eax) };
            }
        };
        place(&mut c, 0x2C);
        for at in placed_slots(w(0xC) == 0) {
            c.ebp = o + at;
            crate::sound::play_timed_slot::play_timed_slot_in(&mut c)?;
        }
        place(&mut c, 0x214);
        c.ebp = o + 0x220;
        crate::sound::play_timed_slot::play_timed_slot_in(&mut c)
    }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    turn_entry(r, car_slot_sounds_in)
}
