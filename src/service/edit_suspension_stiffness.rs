//! `service::edit_suspension_stiffness` (0x46BA6C): the keys while the suspension stiffness is being edited
//! (edit page 6): step down, step up, confirm.
//!
//! In:  ebx = 1 to confirm without taking a key
//! Out: every register preserved
//! What confirming does, and what is refused, is in `pages.rs` (`change_page`).
//! The copy's calls are relocated (not its one near `jmp`, at 0x6F, which stays inside the
//! copy).
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::Refused;
use crate::service::pages::{SUSPENSION_STIFFNESS, change_page};

// ---- Rust version ----

pub fn edit_suspension_stiffness(confirm: bool) -> Result<(), Refused> {
    change_page(crate::menu::owner::menus(), SUSPENSION_STIFFNESS, confirm)
}
