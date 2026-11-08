//! `install::item_action` (0x414A4B): Enter on an action item (kind 6): its action
//! (`[esi+8]`), if it has one.
//!
//! Takes the item.
//! Caller `install::item_enter`.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments.

use crate::errors::Refused;
use crate::install::installer::item_action_at;
use crate::menu::list_handle::ListItem;

/// Enter on an action item; the original's zero after it (the action's, else that of
/// `cmp [esi+8], 1`).
pub fn item_action(
    install: &'static crate::install::owner::Install,
    item: ListItem,
) -> Result<bool, Refused> {
    let action = item.target();
    // Converted: `rc2k_core::install::items::has_action`.
    if rc2k_core::install::items::has_action(action) {
        return item_action_at(install, action);
    }
    Ok(action == 1)
}
