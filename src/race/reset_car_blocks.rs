//! `race::reset_car_blocks` (0x43658A): the eight car blocks set for a race: +0x25C 1 for
//! the race's cars (`g_raceCars`) else 0, +0x38 1, +0x8C `dword_601430` x 100, 22 more
//! dwords 0; each block's +0x24 its kind (`g_raceCarKinds` and the seven after).
//!
//! In:  nothing
//! Out: eax = `dword_601430` x 100, ecx = 8, esi = the last block; the rest preserved
//!
//! See `race/race_cars.rs`. Callers: `sub_43603C` (0x43623A) and its four set-ups
//! (`sub_43645B`, `sub_43647F`, `race::set_up_race_cars`, `sub_4364BD`). Shadow mode: the
//! Rust version, the blocks put back, the original's, compared. Self-test: the game's
//! blocks filled with a pattern, 0 .. 9 and 0x80000000 cars, both versions, put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, ECX, ESI, Regs};
use crate::race::race_cars::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { reset_car_blocks(r) };
    Ok(Left::nothing()
        .with(EAX, g.eax)
        .with(ECX, g.ecx)
        .with(ESI, g.esi))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
