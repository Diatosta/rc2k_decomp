//! `physics::manual_gearbox_call` (0x43C2A8): `call physics::manual_gearbox; retn`.
//!
//! In:  nothing
//! Out: as `manual_gearbox` leaves them
//!
//! Caller: `physics::gearbox_step` (0x43C1AD).
//! Refuses what `manual_gearbox` refuses.
//! Self-test: `physics/gearbox.rs` (`test_gearbox`).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::gearbox::guarded;
use crate::physics::manual_gearbox::manual_gearbox_in;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The Rust version.
pub unsafe fn manual_gearbox_call(entry: &Regs) -> Result<Left, Refused> {
    guarded(entry, &manual_gearbox_in)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { manual_gearbox_call(r) }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
