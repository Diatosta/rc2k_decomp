//! `menu::clear_standings_a` (0x4A5A14): each entry of standings table A (see `menu::standings`)
//! given +0x44 0.
//!
//! In:  nothing
//! Out: every register kept
//!
//! Callers: the event drivers (`sub_4A0E60`, ..). Refuses a table without its end entry, before
//! anything.

use crate::errors::Refused;
use crate::menu::standings::*;

/// The Rust version (see the header): the registers it leaves.
pub fn clear_standings_a() -> Result<(), Refused> {
    let n = walked_a()?;
    with_a(|t| t.entries[..n].iter_mut().for_each(|e| e.head.figure = 0))?;
    Ok(())
}
