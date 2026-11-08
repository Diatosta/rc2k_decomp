//! `install::item_enter` (0x4147FE): Enter on an item, by its kind (`[esi] & 0xF`): 1 a link
//! (`install::item_link`), 4 an edit begun (`install::start_edit`), 6 an action
//! (`install::item_action`).
//!
//! Takes the item, list. Returns the kind.
//! Caller `install::page_key`, reading zero.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments.

use crate::errors::Refused;
use crate::menu::list_handle::ListItem;
use rc2k_core::install::items::EnterWay;

/// Enter on `item`; whether it was taken (the original's zero).
pub fn item_enter(
    install: &'static crate::install::owner::Install,
    item: ListItem,
) -> Result<bool, Refused> {
    let kind = item.kind();
    // Converted: `rc2k_core::install::items::EnterWay`.
    match EnterWay::of(kind) {
        Some(EnterWay::Link) => crate::install::item_link::item_link(install, item),
        Some(EnterWay::StartEdit) => {
            crate::install::start_edit::start_edit(install, item);
            Ok(true)
        }
        Some(EnterWay::Action) => crate::install::item_action::item_action(install, item),
        // cmp bl, 6
        None => Ok(kind == 6),
    }
}
