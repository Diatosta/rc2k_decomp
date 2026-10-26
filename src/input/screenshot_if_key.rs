//! `gfx::screenshot_if_key` (0x43B573): a screenshot while key 0x52 is down.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `input/keys.rs`. One caller, `input::end_frame`. `gfx::take_screenshot` is not
//! ported (a call into the game, section 15). Shadow mode runs the original alone (it
//! may take a screenshot). Self-test: a copy whose `take_screenshot` is a stand-in
//! counting, the key up and down.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::input::keys::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(_: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { screenshot_if_key(&mut game_screenshot) };
    Ok(())
}
