//! `install::item_link` (0x414A28): Enter on a link item (kind 1): its action (`[item+8]`,
//! given its `[item+0Ch]`) if it has one; else, unless its next list (`[item+1]`) is negative,
//! that number into `list_answer` and the page left (`g_listLeave` 1).
//!
//! Takes the item.
//! Caller `install::item_enter`.

use crate::errors::Refused;
use crate::install::installer::item_action_at;
use crate::menu::list_handle::ListItem;
use rc2k_core::install::items::{self, LinkWay};

/// Enter on a link; the original's zero after it (the action's, else that of `test bl, bl`
/// on the next list).
pub fn item_link(
    install: &crate::install::owner::Install,
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
