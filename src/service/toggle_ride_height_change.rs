//! `service::toggle_ride_height_change` (0x46C0F0): flips the flag of the highlighted menu item
//! and, if the flag is then not 1, puts the chosen ride height (`dword_75ACA8`)
//! back to the installed value (`dword_75ACAC`).
//!
//! Refuses a highlighted item past the flag table. "Toggle change" is a reading of that.

use crate::service::setup_values::Change;

// ---- Rust version: `setup_values::toggle_change` with this ----

pub const CHANGE: Change = Change {
    chosen: &crate::menu::owner::MENUS.screens.state.driver_stats.setups[2].chosen,
    installed: &crate::menu::owner::MENUS.screens.state.driver_stats.setups[2].installed,
};
