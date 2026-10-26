//! `drive::record_inputs_if` (0x46008D): `drive::record_inputs` while recording a replay
//! (`dword_5C5CE0` 1) for the cars below `dword_63DB94`.
//!
//! In:  nothing
//! Out: while recording eax `dword_63DB94`, then as `record_inputs` leaves the registers
//!      when called; no flags read
//!
//! Caller: `drive::replay_step` (`sub_460B2F`).
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back): the mode, cars and counts.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, Regs};
use crate::physics::flow::g;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The Rust version.
// Converted: `rc2k_core::drive::replay_frames::records_inputs`.
pub unsafe fn record_inputs_if() -> Left {
    let Some(records) =
        rc2k_core::drive::replay_frames::records_inputs(g(0x5C5CE0), g(0x71B3F8), || g(0x63DB94))
    else {
        return Left::nothing();
    };
    // The compare's `mov eax, dword_63DB94` stays when nothing else writes eax.
    let mut left = Left::nothing().with(EAX, g(0x63DB94));
    if records {
        let called = unsafe { crate::drive::record_inputs::record_inputs() };
        for &(reg, v) in &called.regs {
            left = left.with(reg, v);
        }
    }
    left
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { record_inputs_if() })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
