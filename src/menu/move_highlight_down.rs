//! `menu::move_highlight_down` (0x474715): moves the highlight along the items' link at +0x28, on
//! key 0x6B.
//!
//! In:  nothing (edi is the menu at its one call, and is not used)
//! Out: every register preserved
//! It goes on along the link until it reaches an item that can be chosen. "Down" is a reading.
//! The rules, the click and the item's callback (+0x38) are in `menu/navigation.rs`.
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

// ---- Rust version: `navigation::move_highlight` with `Direction::Down` ----

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::menu::navigation::outside_entry(r, &|r: &Regs, out: &mut dyn Outside| {
        unsafe { move_highlight(Direction::Down, r, out) }?;
        Ok(Left::nothing())
    })
}
