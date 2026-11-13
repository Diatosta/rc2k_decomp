//! Fills the levels' best scores (`saves::best_times::LEVEL_SCORES`) with made-up entries.
//!
//! The table (0x15E0 bytes) is seven groups of ten entries of 0x50 bytes. In each group the
//! entries get 100,000 and then 0xAB3 less each time at +0x14, and a random driver
//! (`saves::random_driver`, a number under 0xAA): the driver's class to the entry's +0x1C,
//! nationality to +8, car to +0x10, the id of the name's string to +0xC and the name (without
//! a NUL) to +0x20. What the table is has not been worked out; "class times" is a guess.
//! One caller, `menu::set_up_front_end`, when `var\hst.dat` was not loaded. Refuses a missing
//! table and a name that does not fit.

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::race::owner::Race;
use crate::saves::best_times::LEVEL_SCORES;
use crate::saves::default_times::{GameSource, to_nowhere};
use rc2k_core::saves::defaults::class_times;

pub fn default_class_times(
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    menus: &Menus,
    race: &Race,
) -> Result<(), Refused> {
    if !LEVEL_SCORES.is_taken() {
        return Err(crate::errors::BadInput("no table").into());
    }
    let entries = class_times(&mut GameSource {
        log: &mut to_nowhere,
        so,
        menus,
        race,
    })?;
    LEVEL_SCORES.with(|t| rc2k_core::saves::entries::fill_made_up(t, &entries))?;
    Ok(())
}
