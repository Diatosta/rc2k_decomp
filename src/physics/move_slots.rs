//! `physics::move_slots` (0x462206): `dword_71BE88` = the number of the eight
//! `g_raceSlots` (0x78 bytes each) in use (+0 not 0), each moved by
//! `physics::move_slot` (inside `pusha` / `popa`).
//!
//! In:  nothing
//! Out: esi past the slots, ecx 8; the rest kept; no flags read (`retn`, then `sub_45E9CA`
//!      returns)
//!
//! Caller: `sub_45E9CA` (0x45E9CF), from race init's loop and `sub_460B2F`.
//! Refuses (before anything) what `move_slot` refuses for a slot in use.
//! Self-test: in a race only: a copy of the original whose slots and count are the
//! test's (its calls to the game's `move_slot`) against the Rust version, slots in use
//! and not; every game variable `move_slot` writes saved and put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{ECX, ESI, Regs, game_addr, rva};
use crate::physics::move_slot::{SLOT_LEN, move_slot_at, slot_refusal};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

const SLOTS_RVA: usize = rva(0x71BAC8); // g_raceSlots
const COUNT_RVA: usize = rva(0x71BE88); // dword_71BE88

/// The Rust core: the slots at `slots`, the count at `count`.
pub unsafe fn move_slots_at(slots: u32, count: u32, r: &Regs) -> Result<Left, Refused> {
    unsafe {
        for k in 0..8 {
            let s = slots + SLOT_LEN * k;
            if (s as *const u32).read_unaligned() != 0
                && let Some(why) = slot_refusal(s)
            {
                return Err(crate::errors::BadInput(why).into());
            }
        }
        let mut n = 0u32;
        for k in 0..8 {
            let s = slots + SLOT_LEN * k;
            if (s as *const u32).read_unaligned() != 0 {
                n += 1;
                (count as *mut u32).write_unaligned(n);
                let mut g = Regs {
                    esi: s,
                    ecx: k,
                    ..*r
                };
                move_slot_at(&mut g)?;
            }
        }
        (count as *mut u32).write_unaligned(n);
        Ok(Left::nothing().with(ESI, slots + 8 * SLOT_LEN).with(ECX, 8))
    }
}

/// The Rust version.
pub unsafe fn move_slots(r: &Regs) -> Result<Left, Refused> {
    unsafe { move_slots_at(game_addr(SLOTS_RVA) as u32, game_addr(COUNT_RVA) as u32, r) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { move_slots(r) }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
