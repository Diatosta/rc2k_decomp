//! `service::toggle_tyre_change` (0x46C0AE): flips the flag of the highlighted menu item
//! and, if the flag is then not 1, puts the chosen tyre (`setups[1].chosen`)
//! back to the installed value (`setups[1].installed`).
//!
//! Refuses a highlighted item past the flag table. "Toggle change" is a reading of that.

use crate::service::setup_values::Change;

// ---- Rust version: `setup_values::toggle_change` with this ----

pub const CHANGE: Change = Change {
    chosen: &crate::menu::owner::MENUS
        .unchecked()
        .screens
        .state
        .driver_stats
        .setups[1]
        .chosen,
    installed: &crate::menu::owner::MENUS
        .unchecked()
        .screens
        .state
        .driver_stats
        .setups[1]
        .installed,
};
