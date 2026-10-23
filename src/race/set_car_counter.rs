//! `race::set_car_counter` (0x43A96C): car block esi's +0x71C eax, +0x730 0, +0x72C 0x12C.
//!
//! In:  esi = the block, eax = the value
//! Out: every register preserved
//!
//! See `race/frame.rs`. Six callers in `sub_43A649` (a `jmp` follows each). Shadow mode:
//! the Rust version, the dwords put back, the original's, compared. Self-test: a block of
//! the test's, values.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { set_car_counter(r.esi, r.eax) };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
