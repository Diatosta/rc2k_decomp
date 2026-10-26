//! `menu::clear_standings_b` (0x4A5A2F): each entry of standings table B (see `menu::standings`)
//! given +0x70 0.
//!
//! In:  nothing
//! Out: every register kept
//!
//! Callers: the event drivers (`sub_4A0E60`, ..). Refuses a table without its end entry, before
//! anything.

use crate::errors::Refused;
use crate::memory::blocks::Place;
use crate::menu::standings::*;

/// The Rust version (see the header): the registers it leaves.
pub fn clear_standings_b() -> Result<(), Refused> {
    let first = Place::Owned(TABLE_B_RVA.need("no standings table")?);
    let all = entries(first, B_ENTRY)?;
    // Converted: `rc2k_core::menu::standings::B_FIGURE`, stored in each entry.
    use rc2k_core::menu::standings::B_FIGURE;
    with_table(first, B_ENTRY, all.len() as u32, |t| {
        t.store_all(&[(B_FIGURE, 0)])
    })?;
    Ok(())
}
