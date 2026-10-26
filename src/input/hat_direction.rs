//! `input::hat_direction` (0x408992): the device's hat (or buttons standing for it) as a
//! code in the second byte: for a device (+4) whose hat kind (+0x418) is not 0, the state's
//! buttons 6, 7, 1, 2 (+0x35, +0x36, +0x30, +0x31) as 0x100 .. 0x400, else its hat (+0x20)
//! straight on (0) 0x500, 270° (27000) 0x600, 180° 0x700, 90° 0xF00, else 0; without a
//! device or with kind 0, 0xE00 (none). (Code after the first `retn` that nothing reaches
//! reads buttons 1 and 2 as 1 and 2.)
//!
//! In:  esi = the device's record
//! Out: eax = the code; the rest kept; no flags read (the caller `or`s it)
//!
//! Caller: `input::read_device`. Self-test: the original and the Rust version on states by
//! lot and at each case, with and without a device, each hat kind.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, Regs};
use crate::input::devices::{joy_state, record_bytes};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The Rust version (see the header).
pub fn hat_direction(record: u32) -> u32 {
    // Converted: `rc2k_core::input::devices::hat_direction`.
    rc2k_core::input::devices::hat_direction(&record_bytes(record), &joy_state())
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, hat_direction(r.esi)))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
