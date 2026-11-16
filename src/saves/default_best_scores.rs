//! Fills the table of best scores with made-up entries.
//!
//! The level scores table (0xBB8 bytes) is five groups of ten entries of 0x3C bytes. In each group
//! the entries get 50, 49, .. 41 at +0x38 and a driver each: a random number under 0xB3 is drawn;
//! for the groups 0 .. 3 it is thrown away and the driver is a random number under
//! `CLASS_DRIVER_COUNTS[group]` plus `CLASS_FIRST_DRIVERS[group]`, for group 4 it is the driver.
//! From the driver's record: its class to the entry's +0x10, nationality to +8, car to +0xC, and
//! the name (without a NUL) to +0x18. Refuses a missing table, a group without drivers, a driver
//! past the 180, and a name that does not fit.

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::race::owner::Race;
use crate::saves::best_times::BestTables;
use crate::saves::default_times::{GameSource, to_nowhere};
use rc2k_core::saves::defaults::best_scores;

pub fn default_best_scores(
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    menus: &Menus,
    race: &Race,
    tables: &BestTables,
) -> Result<(), Refused> {
    if !tables.scores.is_taken() {
        return Err(crate::errors::BadInput("no table of best scores").into());
    }
    let counts = rc2k_core::saves::defaults::CLASS_DRIVER_COUNTS;
    let firsts = rc2k_core::saves::defaults::CLASS_FIRST_DRIVERS;
    let entries = best_scores(
        &mut GameSource {
            log: &mut to_nowhere,
            so,
            menus,
            race,
        },
        counts,
        firsts,
    )?;
    tables
        .scores
        .with(|t| rc2k_core::saves::scores::fill_made_up(t, &entries))?;
    Ok(())
}
