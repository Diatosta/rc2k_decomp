//! `race::set_race_screen_mode` (0x404830): `gfx::set_screen_mode` (0x404B63, the
//! platform's: called in the game) with eax `g_raceScreenMode`; `pusha` / `popa`.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/race_shell.rs`. Callers: `sub_4170E0`, `sub_41CDC2`, `sub_4843EC` (each calls
//! on). Shadow mode runs the original alone (it sets the screen's mode). Self-test: a copy
//! whose call goes to a stand-in that notes it, against the Rust version on the same.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::race_shell::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { set_race_screen_mode(r, &mut game_screen_mode) };
    Ok(())
}
