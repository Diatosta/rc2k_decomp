//! `menu::choose_key_8` (0x46837B): start redefining the key at place 8.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `menu/choose_key.rs`. Compared in shadow mode. Self-test: three settings.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::menu::choose_key::choose_entry;

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    choose_entry(r, 8)
}
