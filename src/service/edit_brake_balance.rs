//! `service::edit_brake_balance` (0x46BCD8): the keys while the brake balance is being
//! edited (edit page 4): step down, step up, and confirm, which leaves edit mode.
//!
//! In:  nothing
//! Out: every register preserved
//! The value is changed in the player as the keys come. Refuses a missing player.
//! The copy's calls are relocated.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::Refused;
use crate::service::pages::{BRAKE_BALANCE, simple_page};

// ---- Rust version ----

pub fn edit_brake_balance() -> Result<(), Refused> {
    simple_page(crate::menu::owner::menus(), BRAKE_BALANCE)
}
