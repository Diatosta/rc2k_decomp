//! `menu::choose_key_6` (0x468351): start redefining the key at place 6.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `menu/choose_key.rs`.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::framework::Regs;
use crate::menu::choose_key::choose_entry;

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    choose_entry(r, 6)
}
