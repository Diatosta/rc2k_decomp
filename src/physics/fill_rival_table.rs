//! Fills the 90 rival times: for each c, `rival_times[c]` = 12000 + 1000 c, `rival_cars[c]` = c
//! % 23, `rival_names[c]` = 0x5A5 + c (a string id: the rival's name); then, when `rivals`
//! points at a table of 90 records of 0x50 bytes, each record's name, car and time go into
//! `rival_names`, `rival_cars` and `rival_times` instead.
use crate::errors::Refused;
use crate::physics::split_tables_store::SplitTables;
use crate::race::vars_store::RaceVars;
use rc2k_core::physics::splits::{RIVALS, default_rival};
use rc2k_core::saves::name_id::NameId;

// Core: `rc2k_core::physics::splits::default_rival`.
pub fn fill_rival_table(
    split_tables: &mut SplitTables,
    vars: &RaceVars,
    tables: &crate::saves::best_times::BestTables,
) -> Result<(), Refused> {
    for c in 0..RIVALS as u32 {
        let (time, car, name) = default_rival(c);
        split_tables.rival_times[c as usize] = time;
        split_tables.rival_cars[c as usize] = car;
        split_tables.rival_names[c as usize] = NameId::String(name);
    }
    // The standings summary (`RACE_STATE.settings.rivals` is 1 while it is there).
    if vars.rivals.get() == 0 {
        return Ok(());
    }
    let rows = tables.summary.with(|t| t.rows)?;
    for (c, r) in rows.iter().take(RIVALS).enumerate() {
        split_tables.rival_names[c] = r.name_id;
        split_tables.rival_cars[c] = r.car;
        split_tables.rival_times[c] = r.time;
    }
    Ok(())
}
