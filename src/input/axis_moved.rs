//! `input::axis_moved` (0x40939F): with eax 0 the axes' rest values taken (after 250 ms);
//! else which of the device's axes moved past 0x80 from rest, and which way (see
//! `input/axes.rs`).
//!
//! In:  eax = 0 to take the rest values, else anything
//! Out: carry when none (or after taking them); else eax the axis (+0x10 up); the rest kept
//!      (ZF too is the original's `test eax, eax`; no caller reads it)
//!
//! Callers: `screens::learn_axis` (two), four of the controller pages' loops. Shadow mode:
//! by state (the rest values) and the carry; with eax 0 the original alone (it sleeps).
//! Self-test: test axes and devices, each axis each way, none moved.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, Regs};
use crate::input::axes::axis_moved;
use crate::screens::harness::Left;

fn rust(r: &Regs) -> Result<Left, Refused> {
    let m = unsafe { axis_moved(r.eax) };
    // No caller reads ZF (they test the carry or go on with a `mov`).
    Ok(Left::carry(m.carry).with(EAX, m.eax))
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::service::harness::state_entry(r, &rust)
}
