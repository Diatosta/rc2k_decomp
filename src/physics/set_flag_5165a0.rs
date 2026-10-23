//! `physics::set_flag_5165a0` (0x45D16D): `dword_5165A0` 1 when the current car's entry in
//! `dword_641680` is not -1. See `physics/listed_car_flags.rs` (the core, the test).
//!
//! In:  nothing
//! Out: eax the entry; the rest kept
//!
//! Caller: `sub_436AFB`.
//! Refuses (before anything) a current car past 7.
//! Self-test: `listed_car_flags::test_twin`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::listed_car_flags::set_flag;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

const FLAG: usize = 0x5165A0; // dword_5165A0

/// The Rust version.
pub unsafe fn set_flag_5165a0() -> Result<Left, Refused> {
    unsafe { set_flag(FLAG) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { set_flag_5165a0() }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
