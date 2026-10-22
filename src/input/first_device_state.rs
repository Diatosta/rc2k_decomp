//! `input::first_device_state` (0x409391): `input::device_state` of `g_inputDevices` (see
//! `input/axes.rs`).
//!
//! In:  nothing
//! Out: ecx, edx, eax, ebx the first device's four values; the rest kept
//!
//! Caller: `screens::read_controller`. Shadow mode: by state. Self-test: the game's device
//! (skipped without one).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, rva};
use crate::input::axes::first_device_state;
use crate::input::device_state::left;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(_: &Regs) -> Result<Left, Refused> {
    if unsafe { game_read::<u32>(rva(0x4F6EE8)) } == 0 {
        return Err(crate::errors::BadInput("no input device").into());
    }
    Ok(left(unsafe { first_device_state() }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
