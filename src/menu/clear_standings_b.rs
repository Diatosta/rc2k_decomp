//! `menu::clear_standings_b` (0x4A5A2F): each entry of standings table B (see `menu::standings`)
//! given +0x70 0.
//!
//! In:  nothing
//! Out: every register kept
//!
//! Callers: the event drivers (`sub_4A0E60`, ..). Refuses a table without its end entry, before
//! anything.

use crate::errors::Refused;
use crate::menu::standings::*;

/// The Rust version (see the header): the registers it leaves.
pub fn clear_standings_b() -> Result<(), Refused> {
    let n = walked_b()?;
    with_b(|t| t.entries[..n].iter_mut().for_each(|e| e.position = 0))?;
    Ok(())
}
