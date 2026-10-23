//! `race::reset_car_race_state` (0x444C1D): each car block's +0x71C 0xCA, +0x72C 0x12C,
//! +0x720/+0x724/+0x728/+0x730 0, the five dwords +0xA0C .. +0xA1C -1; then four dwords
//! from 0x6D1838 (`dword_6D1838`, `a64`, `dword_6D1844`, `a32`) spaces and four from
//! 0x6D1850 (`dword_6D1850`, `a64_0`, `dword_6D185C`, `a32_0`) -1.
//!
//! In:  nothing
//! Out: ebx = 8, edi = the last car block; the rest preserved
//!
//! See `race/race_init.rs`. One caller, `sub_4462B9` (race init). Shadow mode: the Rust
//! version, the blocks and texts put back, the original's, compared. Self-test: the game's
//! blocks and texts filled with values of the test's, both versions, put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EBX, EDI, Regs};
use crate::race::race_init::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { reset_car_race_state(r) };
    Ok(Left::nothing().with(EBX, g.ebx).with(EDI, g.edi))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
