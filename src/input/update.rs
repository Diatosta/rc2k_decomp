//! `input::update` (0x420115): the frame's input, the mouse and the keys pressed.
//!
//! In:  nothing
//! Out: eax the right click, ebx 0x80, ecx 0, edx the last key dword, esi `g_keysNow`,
//!      edi `g_keysBefore`; ebp preserved
//!
//! See `input/keys.rs`. Seven callers (`sub_424001` and the race's and its menus' frame
//! loops). Self-test: the original and the Rust version on the game's input state, set
//! and put back (never with the screenshot key down).
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
        update(r, &mut game_screenshot)
    }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
