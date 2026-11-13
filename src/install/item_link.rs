//! Enter on a link item (kind 1): its action if it has one; else, unless its next list is
//! negative, that number goes into `list_answer` and the page is left. Called by
//! `install::item_enter`.

use crate::errors::Refused;
use crate::install::installer::item_action_at;
use crate::menu::list_handle::ListItem;
use rc2k_core::install::items::{self, LinkWay};

/// Enter on a link; whether it was taken.
pub fn item_link(
    install: &crate::install::owner::Install,
    item: ListItem,
) -> Result<bool, Refused> {
    let action = item.target();
    let mut next = 0;
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
