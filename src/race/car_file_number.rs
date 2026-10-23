//! `race::car_file_number` (0x41F4FD): a car's file number.
//!
//! In:  eax = the car
//! Out: eax = its entry of `dword_5C5928` (the car held to 0 .. 0x21); the rest preserved
//!
//! See `race/mud.rs`. Five callers (`race::load_mud` and the car's loading). Manual patch
//! with a private copy (a `jns` in its first bytes). Self-test: cars -5 .. 0x30.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, Regs};
use crate::race::mud::car_file_number;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, car_file_number(r.eax)))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
