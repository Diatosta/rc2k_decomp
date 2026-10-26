//! `input::device_state` (0x408A49): device esi's four values (see `input/axes.rs`).
//!
//! In:  esi = the device
//! Out: ecx, edx, eax, ebx = +0x2C, +0x30, +0x34, +0x38; the rest kept
//!
//! Caller: `input::first_device_state`. Shadow mode: by state. Self-test: a test device.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, ECX, EDX, Regs};
use crate::input::axes::device_state;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The four as registers.
pub fn left(v: [u32; 4]) -> Left {
    Left::nothing()
        .with(ECX, v[0])
        .with(EDX, v[1])
        .with(EAX, v[2])
        .with(EBX, v[3])
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left(unsafe { device_state(r.esi) }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
