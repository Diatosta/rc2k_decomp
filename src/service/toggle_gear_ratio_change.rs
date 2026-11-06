//! `service::toggle_gear_ratio_change` (0x46C08D): flips the flag of the highlighted menu item
//! and, if the flag is then not 1, puts the chosen gear ratio (`dword_75AC90`, a reading)
//! back to the installed value (`dword_75AC94`).
//!
//! In:  nothing
//! Out: every register preserved
//! Refuses a highlighted item past the flag table. "Toggle change" is a reading of that.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::service::setup_values::Change;

// ---- Rust version: `setup_values::toggle_change` with this ----

pub const CHANGE: Change = Change {
    chosen: &crate::menu::owner::MENUS.screens.state.driver_stats.setups[0].chosen,
    installed: &crate::menu::owner::MENUS.screens.state.driver_stats.setups[0].installed,
};
