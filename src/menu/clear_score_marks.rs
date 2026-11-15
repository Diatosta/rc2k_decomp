//! `menu::clear_score_marks`: the 50 best-score entries (`BestTables::scores`) given `new`
//! (+0x14) 0 and `player` (+4) 0.
//!
//! Callers: the event drivers. Refuses a null table, before anything.

use crate::errors::Refused;
use crate::saves::best_times::BestTables;

pub fn clear_score_marks(tables: &BestTables) -> Result<(), Refused> {
    tables.scores.with(rc2k_core::saves::scores::clear_marks)?;
    Ok(())
}
