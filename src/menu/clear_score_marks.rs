//! `menu::clear_score_marks` (0x4A919A): the 50 best-score entries (`saves::best_times::SCORES`) given `new` (+0x14) 0 and `player` (+4) 0.
//!
//! Callers: the event drivers. Refuses a null table, before anything.

use crate::errors::Refused;

pub fn clear_score_marks() -> Result<(), Refused> {
    // Converted: `rc2k_core::saves::scores::clear_marks`.
    crate::saves::best_times::SCORES.with(rc2k_core::saves::scores::clear_marks)?;
    Ok(())
}
