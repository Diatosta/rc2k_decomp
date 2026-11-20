//! What a build without network play leaves out of the menus (the web build): the main menu has
//! no network item. The item stays in the tables (every other item keeps its number); it is
//! not drawn, the moves along its links pass it by, and the carousel has no entry for it.

use crate::platform::caps;
use rc2k_core::menu::tables::MenuItem;
use rc2k_core::menu::tables_data::ItemFn;

/// The main menu's number for the network item.
const NETWORK_ITEM: u32 = 3;

/// Whether a build without network play leaves the item out of its menu.
pub fn hidden(item: &MenuItem) -> bool {
    !caps::NETWORK && item.action == ItemFn::NetworkRallyLoop
}

/// The main menu's carousel entry of an item: its number less one, and less another for the
/// items after the hidden one, which has no entry.
pub(super) fn carousel_entry(item: u32) -> u32 {
    let entry = item.wrapping_sub(1);
    if !caps::NETWORK && item > NETWORK_ITEM {
        entry - 1
    } else {
        entry
    }
}
