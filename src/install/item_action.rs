//! Enter on an action item (kind 6): its action, if it has one. Called by `install::item_enter`.

use crate::errors::Refused;
use crate::install::installer::item_action_at;
use crate::menu::list_handle::ListItem;

/// Enter on an action item; whether it was taken.
pub fn item_action(
    install: &crate::install::owner::Install,
    list_state: &crate::menu::list_state::ListState,
    item: ListItem,
) -> Result<bool, Refused> {
    let action = item.target();
    if rc2k_core::install::items::has_action(action) {
        return item_action_at(install, list_state, action);
    }
    Ok(action == 1)
}
