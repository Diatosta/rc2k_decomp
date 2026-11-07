//! `service::edit_suspension_stiffness` (0x46BA6C): the keys while the suspension stiffness is being edited
//! (edit page 6): step down, step up, confirm.
//!
//! What confirming does, and what is refused, is in `pages.rs` (`change_page`).

use crate::errors::Refused;
use crate::service::pages::{SUSPENSION_STIFFNESS, change_page};

// ---- Rust version ----

pub fn edit_suspension_stiffness(confirm: bool) -> Result<(), Refused> {
    change_page(crate::menu::owner::menus(), SUSPENSION_STIFFNESS, confirm)
}
