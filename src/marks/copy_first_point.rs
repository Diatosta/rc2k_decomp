//! `marks::copy_first_point` (0x4C1163): `dword_769306` .. `dword_76930E` into
//! `flt_811BDF` .. `flt_811BE7`, and `dword_7692F6` into `dword_811BCF` (`push`/`pop`).
//!
//! In:  nothing
//! Out: every register kept
//!
//! Caller: `sub_4C1148`. Self-test: the four sources by lot, the original and the Rust
//! version compared on the four results; put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};
use crate::screens::harness::Left;
use crate::service::harness::{at, state_entry};

/// (from, to): the point's three floats, then `dword_7692F6`.
const MOVES: [(usize, usize); 4] = [
    (rva(0x769306), rva(0x811BDF)),
    (rva(0x76930A), rva(0x811BE3)),
    (rva(0x76930E), rva(0x811BE7)),
    (rva(0x7692F6), rva(0x811BCF)),
];

/// The Rust version (see the header).
pub unsafe fn copy_first_point() {
    for (from, to) in MOVES {
        unsafe { game_write(to, game_read::<u32>(from)) };
    }
}

/// What it writes.
pub fn regions() -> Vec<crate::service::harness::Region> {
    MOVES.map(|m| at(m.1)).to_vec()
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { copy_first_point() };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
