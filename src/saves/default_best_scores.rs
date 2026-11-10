//! `saves::default_best_scores` (0x47C923): fills the table of best scores with made-up
//! entries.
//!
//! The table at `dword_7556D4` (0xBB8 bytes) is five groups of ten entries of 0x3C bytes.
//! In each group the entries get 50, 49, .. 41 at +0x38 and a driver each
//! (0x47CDA7, a plain function here): a random number under 0xB3 is drawn; for the
//! groups 0 .. 3 it is thrown away and the driver is a random number under
//! `dword_75AB00[group]` plus `dword_75AB10[group]`, for group 4 it is the driver. From
//! the driver's record: its +8 to the entry's +0x10, +0x14 to +8, +0x18 to +0xC, and the
//! name (without a NUL) to +0x18.
//! One caller, `menu::set_up_front_end`, when `var\hst.dat` was not loaded.
//! Refuses a missing table, a group without drivers (the original divides by 0), a
//! driver past the 180, and a name that does not fit.
//! Converted: `rc2k_core::saves::defaults::best_scores`, `rc2k_core::saves::scores::fill_made_up`.

use crate::errors::Refused;
use crate::menu::owner::menus;
use crate::saves::best_times::SCORES;
use crate::saves::default_times::{GameSource, to_nowhere};
use rc2k_core::saves::defaults::best_scores;

// ---- game data ----

pub fn default_best_scores() -> Result<(), Refused> {
    let screens = &menus().screens;
    if !SCORES.is_taken() {
        return Err(crate::errors::BadInput("no table of best scores").into());
    }
    let counts = std::array::from_fn(|g| screens.state.menus.class_counts[g].get());
    let firsts = std::array::from_fn(|g| screens.state.menus.firsts[g].get());
    let entries = best_scores(
        &mut GameSource {
            log: &mut to_nowhere,
        },
        counts,
        firsts,
    )?;
    SCORES.with(|t| rc2k_core::saves::scores::fill_made_up(t, &entries))?;
    Ok(())
}
