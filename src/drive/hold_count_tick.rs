//! `drive::hold_count_tick` (0x460C10): `dword_6235EC` counted down to 0 (a reading: a
//! hold after the start).
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `drive::replay_step`.
//! Self-test: the original and the Rust version at 0, 1 and more.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::flow::{g, p};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The Rust version.
// Converted: `rc2k_core::drive::countdown::hold_tick`.
pub fn hold_count_tick() {
    let was = g(0x6235EC);
    let mut hold = was;
    rc2k_core::drive::countdown::hold_tick(&mut hold);
    if hold != was {
        p(0x6235EC, hold);
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    hold_count_tick();
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
