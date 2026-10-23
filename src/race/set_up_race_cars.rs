//! `race::set_up_race_cars` (0x4364AD): `race::reset_car_blocks`,
//! `race::drop_absent_player`, `race::set_player_car_kinds` (one of `sub_43603C`'s four
//! set-ups, by the kind of race).
//!
//! In:  nothing
//! Out: eax = `dword_601430` x 100, ecx = 8, esi = the last block; the rest preserved
//!
//! See `race/race_cars.rs`. One caller, `sub_43603C` (0x436123). It begins with its call:
//! private copy, its three calls relocated, manual patch. Shadow mode: the Rust version,
//! the blocks put back, the original's, compared. Self-test: the game's blocks filled
//! with a pattern, cars, modes, players and absences of the test's, both versions, put
//! back.
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
    let g = unsafe { set_up_race_cars(r) }?;
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
