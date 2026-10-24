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
pub unsafe fn clear_standings_a() -> Result<(), Refused> {
    let first: u32 = unsafe { crate::framework::game_read(TABLE_A_RVA) };
    let all = entries(first, A_ENTRY)?;
    // Converted: `rc2k_core::menu::standings::A_FIGURE`, stored in each entry.
    use rc2k_core::menu::standings::A_FIGURE;
    unsafe { table(first, A_ENTRY, all.len() as u32) }.store_all(&[(A_FIGURE, 0)]);
    Ok(())
}
