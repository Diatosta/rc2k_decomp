//! `physics::move_slots` (0x462206): `dword_71BE88` = the number of the eight
//! `g_raceSlots` (0x78 bytes each) in use (+0 not 0), each moved by
//! `physics::move_slot` (inside `pusha` / `popa`).
//!
//! In:  nothing
//! Out: esi past the slots, ecx 8; the rest kept; no flags read (`retn`, then `sub_45E9CA`
//!      returns)
//!
//! Caller: `sub_45E9CA` (0x45E9CF), from race init's loop and `sub_460B2F`. Refuses (before
//! anything) what `move_slot` refuses for a slot in use.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::physics::move_slot::move_slot;
use crate::race::owner::Race;
use crate::race::work_store::RACE_WORK;

/// The Rust version: the eight slots, those in use moved, `slot_count` the number done so far.
pub fn move_slots(race: &Race) -> Result<(), Refused> {
    let slots = &RACE_WORK.race_slots;
    let mut n = 0u32;
    for slot in slots {
        if slot.part.get() != 0 {
            n += 1;
            RACE_WORK.slot_count.set(n);
            move_slot(race, slot)?;
        }
    }
    RACE_WORK.slot_count.set(n);
    Ok(())
}
