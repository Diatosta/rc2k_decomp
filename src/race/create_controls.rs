//! `race::create_controls` (0x40941A): `dword_4F7DE4` 3 (`race::load_stage`'s
//! "createcontrols"; what the 3 stands for is not read).
//!
//! In:  nothing
//! Out: every register preserved
//!
//! One caller, `race::load_stage`. Self-test: the variable from 0, put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_write, rva};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

const CONTROLS_RVA: usize = rva(0x4F7DE4); // dword_4F7DE4

pub fn create_controls() {
    unsafe { game_write(CONTROLS_RVA, 3u32) };
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    create_controls();
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
