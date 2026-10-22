//! `input::request_device_release` (0x409386): `g_inputCommand` 2: the input thread ends the
//! effects and gives both devices back (`input::device_command`). The same store as
//! `race::release_controls` (0x409425).
//!
//! In:  nothing
//! Out: every register kept
//! Caller: `menu::release_controller`. Self-test: the variable from 0, 1 and 3, put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_write};
use crate::input::input_tick::COMMAND_RVA;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The Rust version (see the header).
pub fn request_device_release() {
    unsafe { game_write(COMMAND_RVA, 2u32) };
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    request_device_release();
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
