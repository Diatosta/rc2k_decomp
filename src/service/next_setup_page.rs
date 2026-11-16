//! Shows the next page of the setup slot list: adds 1 to `setup_slot_page`, and from 3 on goes
//! back to 0. The comparison is signed, so a negative page just goes up by one.

use crate::menu::owner::Menus;

pub fn next_setup_page(menus: &Menus) {
    let tables = &menus.tables;
    tables
        .pages
        .setup_slot_page
        .set(
            rc2k_core::service::setups::next_page(tables.pages.setup_slot_page.get() as i32) as u32,
        );
}
