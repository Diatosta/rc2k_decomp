//! `input::end_frame` (0x43AA91): the screenshot key looked at (with screenshots on),
//! two of the frame's dwords cleared.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `input/keys.rs`. One caller, `input::update`. Shadow mode runs the original alone
//! (it may take a screenshot). Self-test: a copy whose `screenshot_if_key` is a stand-in
//! counting, screenshots off and on, the key up and down.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::input::keys::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(_: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { end_frame(&mut game_check) };
    Ok(())
}
