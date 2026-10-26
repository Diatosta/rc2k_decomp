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
use crate::physics::move_slot::{SLOT_LEN, move_slot_at, slot_refusal};
use crate::race::work_store::RACE_WORK;

/// The Rust core: the slots at `slots`, the count at `count`.
pub unsafe fn move_slots_at(slots: u32, count: u32) -> Result<(), Refused> {
    unsafe {
        for k in 0..8 {
            let s = slots + SLOT_LEN * k;
            if (crate::pointers::host(s) as *const u32).read_unaligned() != 0
                && let Some(why) = slot_refusal(s)
            {
                return Err(crate::errors::BadInput(why).into());
            }
        }
        let mut n = 0u32;
        for k in 0..8 {
            let s = slots + SLOT_LEN * k;
            if (crate::pointers::host(s) as *const u32).read_unaligned() != 0 {
                n += 1;
                (crate::pointers::host(count) as *mut u32).write_unaligned(n);
                move_slot_at(s)?;
            }
        }
        (crate::pointers::host(count) as *mut u32).write_unaligned(n);
        Ok(())
    }
}

/// The Rust version.
pub unsafe fn move_slots() -> Result<(), Refused> {
    unsafe {
        move_slots_at(
            crate::pointers::host_number(RACE_WORK.race_slots.as_ptr()) as u32,
            RACE_WORK.slot_count.addr() as u32,
        )
    }
}
