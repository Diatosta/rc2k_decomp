//! `service::edit_tyre` (0x46BD44): the keys while the tyre is being edited
//! (edit page 2): step down, step up, confirm.
//!
//! In:  ebx = 1 to confirm without taking a key
//! Out: every register preserved
//! What confirming does, and what is refused, is in `pages.rs` (`change_page`).
//! The copy's calls are relocated.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::Refused;
use crate::service::pages::{TYRE, change_page};

// ---- Rust version ----

pub fn edit_tyre(confirm: bool) -> Result<(), Refused> {
    change_page(crate::menu::owner::menus(), TYRE, confirm)
}
