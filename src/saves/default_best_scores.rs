//! `saves::default_best_scores` (0x47C923): fills the table of best scores with made-up
//! entries.
//!
//! The level scores table (0xBB8 bytes) is five groups of ten entries of 0x3C bytes.
//! In each group the entries get 50, 49, .. 41 at +0x38 and a driver each
//! (0x47CDA7, a plain function here): a random number under 0xB3 is drawn; for the
//! groups 0 .. 3 it is thrown away and the driver is a random number under
//! `class_counts[group]` plus `firsts[group]`, for group 4 it is the driver. From
//! the driver's record: its +8 to the entry's +0x10, +0x14 to +8, +0x18 to +0xC, and the
//! name (without a NUL) to +0x18.
//! One caller, `menu::set_up_front_end`, when `var\hst.dat` was not loaded.
//! Refuses a missing table, a group without drivers (the original divides by 0), a
//! driver past the 180, and a name that does not fit.
//! Converted: `rc2k_core::saves::defaults::best_scores`, `rc2k_core::saves::scores::fill_made_up`.

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::race::owner::Race;
use crate::saves::best_times::SCORES;
use crate::saves::default_times::{GameSource, to_nowhere};
use rc2k_core::saves::defaults::best_scores;

// ---- game data ----

pub fn default_best_scores(
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    menus: &Menus,
    race: &Race,
) -> Result<(), Refused> {
    let screens = &menus.screens;
    if !SCORES.is_taken() {
        return Err(crate::errors::BadInput("no table of best scores").into());
    }
    let counts = std::array::from_fn(|g| screens.state.menus.class_counts[g].get());
    let firsts = std::array::from_fn(|g| screens.state.menus.firsts[g].get());
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
    SCORES.with(|t| rc2k_core::saves::scores::fill_made_up(t, &entries))?;
    Ok(())
}
