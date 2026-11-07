//! `service::edit_tyre` (0x46BD44): the keys while the tyre is being edited
//! (edit page 2): step down, step up, confirm.
//!
//! What confirming does, and what is refused, is in `pages.rs` (`change_page`).

use crate::errors::Refused;
use crate::service::pages::{TYRE, change_page};

// ---- Rust version ----

pub fn edit_tyre(confirm: bool) -> Result<(), Refused> {
    change_page(crate::menu::owner::menus(), TYRE, confirm)
}
