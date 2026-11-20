//! Shows the next page of the setup slot list: adds 1 to `setup_slot_page`, and from 3 on goes
//! back to 0. The comparison is signed, so a negative page just goes up by one.

use crate::menu::owner::Menus;

pub fn next_setup_page(menus: &Menus) {
    let tables = &menus.tables;
    tables.pages.write().setup_slot_page =
        rc2k_core::service::setups::next_page(tables.pages.read().setup_slot_page as i32) as u32;
}
