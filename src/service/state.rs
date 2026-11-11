//! Game variables the service area's functions share.

use crate::errors::BadInput;
use crate::menu::owner::Menus;
use crate::menu::owner::{Screens, Tables, menus};
use crate::session::{read_all, write_all};
use crate::system::checked_reads::refuse;

/// The current player (`Player`), or None if the index is past the four.
pub fn current_player(menus: &Menus) -> Option<crate::menu::standings::PlayerCell<'_>> {
    crate::menu::player::player(menus, menus.screens.event.current_player.get())
}

/// The flag of menu item `item` in the repair flag table (the item not checked against
/// the table: a read past it reads on, as the original's).
pub fn repair_flag(menus: &Menus, item: u32) -> u32 {
    repair_flag_in(&menus.tables, item)
}

/// `repair_flag` for a caller that holds the tables group.
pub fn repair_flag_in(tables: &Tables, item: u32) -> u32 {
    crate::session::read_on_in(
        &tables.lists,
        &tables.lists.inspection_flags_a,
        item as usize,
        "a repair flag",
    )
    .unwrap_or_else(|e| refuse(e))
}

pub fn edit_mode(menus: &Menus) -> bool {
    menus.screens.state.service_ui.box_shown.is_set()
}

pub fn menu_item() -> u32 {
    menus().screens.loop_flags.item.get()
}

pub fn blink_on(menus: &Menus) -> bool {
    menus.screens.draw_work.blink.is_set()
}

/// Menu items of the service area that have a flag: 0 .. 19. Entry 20 of both flag tables is
/// the -1 that ends them.
pub const ITEM_COUNT: u32 = 20;

/// The highlighted menu item, if it has a flag.
pub fn flagged_item(screens: &Screens) -> Result<u32, BadInput> {
    let item = screens.loop_flags.item.get();
    if item < ITEM_COUNT {
        Ok(item)
    } else {
        Err(BadInput("the highlighted menu item has no repair flag"))
    }
}

pub fn set_repair_flag(tables: &Tables, item: u32, value: u32) {
    crate::session::write_on_in(
        &tables.lists,
        &tables.lists.inspection_flags_a,
        item as usize,
        value,
        "a repair flag",
    )
    .unwrap_or_else(|e| refuse(e));
}

/// The flag table, with its end mark.
pub fn flags(tables: &Tables) -> rc2k_core::service::repairs::Table {
    read_all(&tables.lists.inspection_flags_a)
}

pub fn store_flags(tables: &Tables, t: &rc2k_core::service::repairs::Table) {
    write_all(&tables.lists.inspection_flags_a, *t);
}

/// The mark table, with its end mark.
pub fn marks(tables: &Tables) -> rc2k_core::service::repairs::Table {
    read_all(&tables.lists.inspection_flags_b)
}

pub fn store_marks(tables: &Tables, t: &rc2k_core::service::repairs::Table) {
    write_all(&tables.lists.inspection_flags_b, *t);
}

/// An item's repair time, through its pointer (None: null).
pub fn time_of(menus: &Menus, tables: &Tables, item: usize) -> Option<i32> {
    let key: u32 = tables
        .lists
        .repair_time_ptrs
        .get(item)
        .map(crate::session::DwordLike::get)
        .unwrap_or_else(|| refuse(BadInput("an item past the repair times' pointers")));
    crate::menu::lists_store::repair_time_at(menus, key)
        .unwrap_or_else(|e| refuse(e))
        .map(|time| time.get() as i32)
}

/// The time of every item whose flag is not 0, as the original adds it up. Converted:
/// `rc2k_core::service::repairs::chosen_time`.
pub fn chosen_repairs_time(menus: &Menus, tables: &Tables) -> Result<i32, BadInput> {
    Ok(rc2k_core::service::repairs::chosen_time(
        &flags(tables),
        |item| time_of(menus, tables, item),
    )?)
}

pub fn leave_edit_mode(screens: &Screens) {
    screens.state.service_ui.box_shown.set(false);
}

/// True if menu item `item` has a repair time (a non-null pointer in the table).
pub fn has_repair_time(tables: &Tables, item: u32) -> bool {
    item < ITEM_COUNT && tables.lists.repair_time_ptrs[item as usize].is_set()
}

/// True if the second flag table has its end mark where it belongs.
pub fn marks_ended(tables: &Tables) -> bool {
    rc2k_core::service::repairs::end_of(&marks(tables)).is_some()
}
