//! `install::item_enter` (0x4147FE): Enter on an item, by its kind (`[esi] & 0xF`): 1 a link
//! (`install::item_link`), 4 an edit begun (`install::start_edit`), 6 an action
//! (`install::item_action`).
//!
//! In:  esi the item, edi the list
//! Out: bl the kind; as the callee leaves them, else the flags of `cmp bl, 6`
//! Caller `install::page_key`, reading zero.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::rb;
use rc2k_core::install::items::EnterWay;

/// Enter on `item`; whether it was taken (the original's zero).
pub fn item_enter(item: u32) -> Result<bool, Refused> {
    let kind = rb(item) & 0xF;
    // Converted: `rc2k_core::install::items::EnterWay`.
    match EnterWay::of(kind) {
        Some(EnterWay::Link) => crate::install::item_link::item_link(item),
        Some(EnterWay::StartEdit) => {
            crate::install::start_edit::start_edit(item);
            Ok(true)
        }
        Some(EnterWay::Action) => crate::install::item_action::item_action(item),
        // cmp bl, 6
        None => Ok(kind == 6),
    }
}
