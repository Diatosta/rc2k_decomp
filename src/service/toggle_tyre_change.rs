//! `service::toggle_tyre_change` (0x46C0AE): flips the flag of the highlighted menu item
//! and, if the flag is then not 1, puts the chosen tyre (`dword_75AC9C`)
//! back to the installed value (`dword_75ACA0`).
//!
//! In:  nothing
//! Out: every register preserved
//! Refuses a highlighted item past the flag table. "Toggle change" is a reading of that.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::menu::state_store::MENU_STATE;
use crate::service::setup_values::Change;

// ---- Rust version: `setup_values::toggle_change` with this ----

pub const CHANGE: Change = Change {
    chosen: &MENU_STATE.driver_stats.setups[1].chosen,
    installed: &MENU_STATE.driver_stats.setups[1].installed,
};
