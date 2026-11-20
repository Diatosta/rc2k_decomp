//! `menu::clear_score_marks`: the 50 best-score entries (`BestTables::scores`) given a `mark`
//! of 0 and a `player` of 0.
//!
//! Refuses a null table, before anything.

use crate::errors::Refused;
use crate::saves::best_times::BestTables;

pub(super) fn clear_score_marks(tables: &BestTables) -> Result<(), Refused> {
    tables.scores.with(rc2k_core::saves::scores::clear_marks)?;
    Ok(())
}
