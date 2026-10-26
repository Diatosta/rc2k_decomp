//! `drive::flush_replay_if` (0x460031): `drive::flush_replay` while recording
//! (`dword_5C5CE0` 1) at the last tenth (`dword_6EF7DC` 0x63).
//!
//! In:  nothing
//! Out: as `flush_replay` leaves the registers when called; no flags read
//!
//! Caller: `drive::replay_step`.
//! Refuses what `flush_replay` refuses. Shadow mode runs the original alone (a file
//! write).
//! Self-test: the original and the Rust version off the tenth and with the replay not
//! being written.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::flow::g;
use crate::screens::harness::Left;
use rc2k_core::drive::replay_frames::flush_due;

/// The Rust version.
// Converted: `rc2k_core::drive::replay_frames::flush_due`.
pub unsafe fn flush_replay_if(entry: &Regs) -> Result<Left, Refused> {
    if flush_due(g(0x5C5CE0), g(0x6EF7DC)) {
        return unsafe { crate::drive::flush_replay::flush_replay(entry) };
    }
    Ok(Left::nothing())
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { flush_replay_if(r) }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = rust(r)?;
    left.apply(r);
    Ok(())
}
