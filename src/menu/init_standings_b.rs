//! `menu::init_standings_b`: the 200 entries of standings table B emptied (their
//! first five dwords -1).
//!
//! Refuses a null table, before anything.
use crate::errors::Refused;
use crate::menu::standings::*;

pub fn init_standings_b(tables: &crate::saves::best_times::BestTables) -> Result<(), Refused> {
    // The logic is in `rc2k_core::saves::rankings::empty_b` on the first `ROOM` entries.
    use rc2k_core::menu::standings::ROOM;
    with_b(tables, |t| {
        t.entries[..ROOM as usize]
            .iter_mut()
            .for_each(rc2k_core::saves::rankings::empty_b);
    })?;
    Ok(())
}
