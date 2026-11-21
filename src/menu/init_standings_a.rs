//! `menu::init_standings_a`: the 200 entries of standings table A emptied: `first`
//! and `player` -1, `nation`, `name_id` and `car` 0, `key` / `rally_time` 3600370 (a time beyond
//! any), `group` 0.
//!
//! Refuses a null table, before anything.
use crate::errors::Refused;
use crate::menu::standings::*;

pub fn init_standings_a(tables: &crate::saves::best_times::BestTables) -> Result<(), Refused> {
    // The logic is in `rc2k_core::saves::rankings::empty_a` on the first `ROOM` entries.
    use rc2k_core::menu::standings::ROOM;
    with_a(tables, |t| {
        t.entries[..ROOM as usize]
            .iter_mut()
            .for_each(rc2k_core::saves::rankings::empty_a);
    })?;
    Ok(())
}
