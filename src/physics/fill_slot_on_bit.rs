//! `physics::fill_slot_on_bit` (0x462237): when bit cl of `g_car` +0x298 is set, it is
//! cleared and part 0x50 + 0x14 x `g_currentCar` + cl goes into a race slot
//! (`physics::fill_race_slot`).
//!
//! In:  cl the bit (ecx passed on)
//! Out: eax 1 << cl, or as `fill_race_slot` leaves them; no flags read
//!
//! Callers: `physics::drop_broken_parts` (seven), `sub_4410C8` (four).
//! Refuses what `fill_race_slot` refuses (before anything: the bit is cleared after).
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back): bits set and clear, cars 0 .. 7.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::Regs;
use crate::physics::fill_race_slot::{fill_race_slot_in, g, p, regions};
use crate::screens::harness::Left;
use crate::service::harness::{capture, restore, state_entry};
use rc2k_core::physics::parts::{part_number, take_bit};

const BITS: usize = 0x71A270; // g_car + 0x298

/// The steps on `r` (cl the bit).
// Converted: `rc2k_core::physics::parts::{take_bit, part_number}`.
pub fn fill_slot_on_bit_in(r: &mut Regs) -> Result<(), Refused> {
    let (mask, cleared) = take_bit(g(BITS), r.ecx);
    r.eax = mask;
    let Some(cleared) = cleared else {
        return Ok(());
    };
    let Some(part) = part_number(g(0x71B3F8), r.ecx) else {
        return Err(BadInput("a part past 0xFF").into());
    };
    p(BITS, cleared);
    r.eax = part;
    fill_race_slot_in(r)
}

/// The Rust version.
pub unsafe fn fill_slot_on_bit(entry: &Regs) -> Result<Left, Refused> {
    let all = regions();
    let saved = capture(&all);
    let mut r = *entry;
    match fill_slot_on_bit_in(&mut r) {
        Ok(()) => Ok(crate::race::stage_prefs::left_of(&r)),
        Err(e) => {
            restore(&all, &saved);
            Err(e)
        }
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { fill_slot_on_bit(r) }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
