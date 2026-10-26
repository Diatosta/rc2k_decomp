//! `input::hat_reading` (0x408968): the device's hat as an angle in 256ths: with a device
//! (+4) whose hat kind (+0x418) is 0 and a hat not centred (the state's +0x20,
//! `dword_4F6A48`, its low word not 0xFFFF), the hat's hundredths of degrees x 256 / 36000
//! (`div` of 0x8CA0); else 0.
//!
//! In:  esi = the device's record
//! Out: eax = the angle; ecx = 0x8CA0 and edx the remainder after the `div`; no flags read
//!
//! Caller: `input::read_device` (its bits' top byte). Self-test: the original and the Rust
//! version on angles by lot and centred, with and without a device, each hat kind.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, ECX, EDX, Regs};
use crate::input::devices::{joy_state, record_bytes};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// dword_4F6A48

/// The Rust version (see the header): (eax, the `div`'s ecx and edx when it was reached).
pub fn hat_reading(record: u32) -> (u32, Option<(u32, u32)>) {
    // Converted: `rc2k_core::input::devices::hat_reading`.
    rc2k_core::input::devices::hat_reading(&record_bytes(record), &joy_state())
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let (eax, div) = hat_reading(r.esi);
    let left = Left::nothing().with(EAX, eax);
    Ok(match div {
        Some((ecx, edx)) => left.with(ECX, ecx).with(EDX, edx),
        None => left,
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
