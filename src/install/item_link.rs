//! `install::item_link` (0x414A28): Enter on a link item (kind 1): its action (`[esi+8]`,
//! edx its `[esi+0Ch]`) if it has one; else, unless its next list (`[esi+1]`) is negative,
//! that number into `byte_51D415` and the page left (`g_listLeave` 1).
//!
//! In:  esi the item (edi the list, for the action)
//! Out: with an action as it leaves them; else bl the next list and the flags of
//!      `test bl, bl` (zero for list 0, carry clear)
//! Caller `install::item_enter`.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::item_action_at;
use crate::menu::list_handle::ListItem;
use rc2k_core::install::items::{self, LinkWay};

/// Enter on a link; the original's zero after it (the action's, else that of `test bl, bl`
/// on the next list).
pub fn item_link(
    install: &'static crate::install::owner::Install,
    item: ListItem,
) -> Result<bool, Refused> {
    let action = item.target();
    let mut next = 0;
    // Converted: `rc2k_core::install::items::link`.
    match items::link(action, || {
        next = item.count();
        next
    }) {
        LinkWay::Action => item_action_at(install, action),
        LinkWay::Leave(next) => {
            install.pages.list_answer.set(next);
            install.pages.list_leave.set(1);
            Ok(next == 0)
        }
        LinkWay::Stay => Ok(next == 0),
    }
}
