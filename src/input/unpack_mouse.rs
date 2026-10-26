//! `input::unpack_mouse` (0x4081FC): the mouse word unpacked.
//!
//! In:  nothing
//! Out: ebx = the buttons, ecx = x, edx = y (signed); the rest preserved
//!
//! See `input/keys.rs`. One caller, `input::take_mouse`. Self-test: the original and the
//! Rust version on the game's input state, set and put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::input::keys::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(crate::race::stage_prefs::left_of(&unsafe {
        unpack_mouse(r)
    }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
