//! Enter on an item, by its kind: 1 a link (`install::item_link`), 4 an edit begun
//! (`install::start_edit`), 6 an action (`install::item_action`). Called by `install::page_key`.

use crate::errors::Refused;
use crate::menu::list_handle::ListItem;
use rc2k_core::install::items::EnterWay;

/// Enter on `item`; whether it was taken.
pub fn item_enter(
    install: &'static crate::install::owner::Install,
    item: ListItem,
) -> Result<bool, Refused> {
    let kind = item.kind();
    match EnterWay::of(kind) {
        Some(EnterWay::Link) => crate::install::item_link::item_link(install, item),
        Some(EnterWay::StartEdit) => {
            crate::install::start_edit::start_edit(install, item);
            Ok(true)
        }
        Some(EnterWay::Action) => crate::install::item_action::item_action(install, item),
        None => Ok(kind == 6),
    }
}
