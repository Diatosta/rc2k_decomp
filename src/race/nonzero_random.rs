//! `race::nonzero_random` (0x444C14): `math::next_random`, 1 in place of 0.
//!
//! In:  nothing
//! Out: eax = the number; the rest preserved (the flags are `next_random`'s or `inc`'s;
//!      the one caller does not read them)
//!
//! See `race/race_init.rs`. One caller, `race::seed_car_randoms` (0x444BF6). It begins
//! with its call: private copy, the call relocated, manual patch. Shadow mode: the Rust
//! version, the generator's state put back, the original's, compared. Self-test: the
//! states of `math::next_random`'s test (0 among them, whose next number is 0).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, Regs};
use crate::race::race_init::nonzero_random;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, unsafe { nonzero_random() }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
