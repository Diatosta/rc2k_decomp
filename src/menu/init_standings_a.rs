//! `menu::init_standings_a`: the 200 entries of standings table A emptied: +0
//! and +4 -1, +8 .. +0x10 0, +0x14 / +0x18 3600370 (a time beyond any), +0x1C 0.
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
