//! `race::reset_car_starts` (0x444BBC): each car block's +0x14 and +8 0, or 0x320 while
//! `dword_71C3E0` (the replay) is set; +0x18 and +0x1C 0.
//!
//! In:  nothing
//! Out: eax = that value, ecx = 8, esi = the last car block; the rest preserved
//!
//! See `race/race_init.rs`. One caller, `sub_4462B9` (race init). Shadow mode: the Rust
//! version, the blocks put back, the original's, compared. Self-test: the game's eight
//! blocks filled with values of the test's, `dword_71C3E0` 0, 1 and 0x80000000, both
//! versions, put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, ECX, ESI, Regs};
use crate::race::race_init::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { reset_car_starts(r) };
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
