//! `menu::clear_score_marks` (0x4A919A): the 50 best-score entries (`dword_7556D4`, 0x3C
//! each; see `saves::default_best_scores`) given +0x14 0 and +4 0.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the event drivers. Refuses a null table, before anything.

use crate::errors::{BadInput, Refused};
use crate::menu::champ_store::CHAMP_VIEW;
use crate::menu::standings::*;

const ENTRIES: u32 = 50;

/// The Rust version (see the header): the registers it leaves.
pub fn clear_score_marks() -> Result<(), Refused> {
    let first: u32 = CHAMP_VIEW.best_scores.get();
    if first == 0 {
        return Err(BadInput("no best-score table").into());
    }
    // Converted: `rc2k_core::menu::standings::BEST_MARKS`, stored in each entry.
    table(first, 0x3C, ENTRIES)?.store_all(&rc2k_core::menu::standings::BEST_MARKS);
    Ok(())
}
