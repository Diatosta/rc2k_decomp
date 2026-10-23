//! `race::set_up_mode_4_cars` (0x43647F): `sub_43603C`'s set-up for `dword_62389C` 4:
//! `g_raceCars`, `dword_63DB94`, `dword_63DB98` 1; `race::reset_car_blocks`,
//! `drop_absent_player`, `set_player_car_kinds`.
//!
//! In:  nothing
//! Out: eax = `dword_601430` x 100, ecx = 8, esi = the last block (as `reset_car_blocks`)
//!
//! See `race/race_cars.rs`. One caller, `sub_43603C`. Shadow mode: the Rust version (its steps the
//! Rust versions), the blocks put back, the original's, compared. Self-test: a copy whose
//! calls note themselves, on the game's blocks (filled, put back), against the Rust version
//! on the same.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::race_cars::*;
use crate::race::start_place::left_changed;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { set_up_mode_4_cars(r) }?;
    Ok(left_changed(r, &g))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
