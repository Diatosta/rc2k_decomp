//! `menu::cheat_car_list` (0x480D35): with cheat bit 1 of `g_cheats` set, esi becomes the
//! cheat's car list `off_75A412`; else esi stays.
//!
//! In:  esi a car list
//! Out: esi; carry = the bit (`bt`); the rest kept
//!
//! Callers: the time trial's events, before each car choice. Self-test: each bit value,
//! `g_cheats` put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{ESI, Regs, game_addr, game_read, rva};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

const CHEATS_RVA: usize = rva(0x75870C); // g_cheats
const CHEAT_CARS_RVA: usize = rva(0x75A412); // off_75A412

/// The Rust version: esi and the carry in `r`.
pub fn cheat_car_list(r: &mut Regs) {
    // Converted: `rc2k_core::menu::car_lists::cheat_cars`.
    let on = rc2k_core::menu::car_lists::cheat_cars(unsafe { game_read(CHEATS_RVA) });
    if on {
        r.esi = game_addr(CHEAT_CARS_RVA) as u32;
    }
    r.set_carry(on);
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let mut g = *r;
    cheat_car_list(&mut g);
    Ok(Left::carry(g.carry()).with(ESI, g.esi))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
