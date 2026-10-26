//! `screens::classify_controller` (0x46B27A): the button and the direction held from the
//! controller's four values, into `dword_7559EC` (see `screens/controller.rs`).
//!
//! In:  ecx x, edx y, eax z, ebx the buttons' bits
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Caller: `screens::learn_axis`. Shadow mode: by state. Self-test: buttons none, one and
//! two, each direction held through 32 polls and not; the histories put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::screens::controller::classify_controller;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { classify_controller(r.eax, r.ebx, r.ecx, r.edx) };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
