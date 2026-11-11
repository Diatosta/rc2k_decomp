//! `service::toggle_gear_ratio_change` (0x46C08D): flips the flag of the highlighted menu item
//! and, if the flag is then not 1, puts the chosen gear ratio (`setups[0].chosen`, setup item 0)
//! back to the installed value (`setups[0].installed`).
//!
//! Refuses a highlighted item past the flag table. "Toggle change" names what it does: the item's change flag is flipped.

use crate::service::setup_values::Change;

// ---- Rust version: `setup_values::toggle_change` with this ----

pub const CHANGE: Change = Change {
    chosen: &crate::menu::owner::MENUS
        .unchecked()
        .screens
        .state
        .driver_stats
        .setups[0]
        .chosen,
    installed: &crate::menu::owner::MENUS
        .unchecked()
        .screens
        .state
        .driver_stats
        .setups[0]
        .installed,
};
