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
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use rc2k_core::sound::engine::{computers, placed_slots};

// ---- Rust version ----

/// The timed slots of object `o`, through `request`.
pub fn car_slot_sounds(o: u32, request: u32) -> Result<(), Refused> {
    if o == 0 || request == 0 {
        return Err(BadInput("no object or no request").into());
    }
    let so = unsafe { crate::sound::store::SoundObject::at(o) };
    // Converted: `rc2k_core::sound::engine` (`computers`, `placed_slots`).
    if computers(so.sound_offset.get()) {
        return Ok(());
    }
    let q = request as *mut u32;
    let place = |from: &[crate::session::Float; 3]| {
        for (k, f) in from.iter().enumerate() {
            unsafe { q.add(3 + k).write_unaligned(f.bits()) };
        }
    };
    place(&so.place);
    for at in placed_slots(so.view_flag.get() == 0) {
        crate::sound::play_timed_slot::play_timed_slot(so.slot_at(at).addr(), request)?;
    }
    place(&so.place_b);
    crate::sound::play_timed_slot::play_timed_slot(so.slot_b.addr(), request)
}
