//! `input::find_pressed_keys` (0x420215): the keys that went down this frame.
//!
//! In:  nothing
//! Out: ebx 0x80, ecx 0, eax and edx the last dword's pressed and now; the rest preserved
//!
//! See `input/keys.rs`. One caller, `input::update`. Self-test: the original and the Rust
//! version on the game's input state, set and put back.
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
        find_pressed_keys(r)
    }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
