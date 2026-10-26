//! `menu::clear_score_marks` (0x4A919A): the 50 best-score entries (`dword_7556D4`, 0x3C
//! each; see `saves::default_best_scores`) given +0x14 0 and +4 0.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the event drivers. Refuses a null table, before anything.

use crate::errors::Refused;
use crate::memory::blocks::Place;
use crate::menu::champ_store::CHAMP_VIEW;
use crate::menu::standings::*;

const ENTRIES: u32 = 50;

/// The Rust version (see the header): the registers it leaves.
pub fn clear_score_marks() -> Result<(), Refused> {
    let first = CHAMP_VIEW.best_scores.need("no best-score table")?;
    // Converted: `rc2k_core::menu::standings::BEST_MARKS`, stored in each entry.
    with_table(Place::Owned(first), 0x3C, ENTRIES, |t| {
        t.store_all(&rc2k_core::menu::standings::BEST_MARKS)
    })?;
    Ok(())
}
