//! `menu::move_highlight_right` (0x474766): moves the highlight along the items' link at +0x30, on
//! key 0x69.
//!
//! In:  nothing (edi is the menu at its one call, and is not used)
//! Out: every register preserved
//! One step; while `dword_7554EC` is 1 it first turns the counter `dword_75AA7C` forwards.
//! The rules, the click and the item's callback (+0x40) are in `menu/navigation.rs`.
//! One caller, `menu::handle_keys`. Shadow mode: the original runs for real, then the Rust
//! version without the click and the callback, and the two are compared unless a callback
//! was left out. Self-test: a redirected copy whose click is a stand-in.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::menu::navigation::{Direction, Outside, move_highlight};
use crate::screens::harness::Left;

// ---- Rust version: `navigation::move_highlight` with `Direction::Right` ----

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::menu::navigation::outside_entry(r, &|r: &Regs, out: &mut dyn Outside| {
        unsafe { move_highlight(Direction::Right, r, out) }?;
        Ok(Left::nothing())
    })
}
