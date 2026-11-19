//! What the web build leaves out of the menus: it has no network play, so the main menu has
//! no network item. The item stays in the tables (every other item keeps its number); it is
//! not drawn, the moves along its links pass it by, and the carousel has no entry for it.

use rc2k_core::menu::tables::MenuItem;
use rc2k_core::menu::tables_data::ItemFn;

/// The main menu's number for the network item.
pub const NETWORK_ITEM: u32 = 3;

/// Whether the web build leaves the item out of its menu.
pub fn hidden(item: &MenuItem) -> bool {
    cfg!(web) && item.action == ItemFn::NetworkRallyLoop
}

/// The main menu's carousel entry of an item: its number less one, and less another for the
/// items after the hidden one, which has no entry.
pub fn carousel_entry(item: u32) -> u32 {
    let entry = item.wrapping_sub(1);
    if cfg!(web) && item > NETWORK_ITEM {
        entry - 1
    } else {
        entry
    }
}
