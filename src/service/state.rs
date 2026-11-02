//! Game variables the service area's functions share.

use crate::errors::BadInput;
use crate::menu::draw_work_store::DRAW_WORK;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::lists_store::MENU_LISTS;
use crate::menu::loop_store::LOOP_FLAGS;
use crate::menu::state_store::MENU_STATE;
use crate::session::{read_all, write_all};
use crate::system::checked_reads::refuse;

/// The current player (`Player`), or None if the index is past the four.
pub fn current_player() -> Option<crate::menu::standings::PlayerCell> {
    crate::menu::player::player(EVENT_STATE.current_player.get())
}

/// The flag of menu item `item` in the repair flag table (the item not checked against
/// the table: a read past it reads on, as the original's).
pub fn repair_flag(item: u32) -> u32 {
    crate::session::read_on_in(
        &MENU_LISTS,
        &MENU_LISTS.inspection_flags_a,
        item as usize,
        "a repair flag",
    )
    .unwrap_or_else(|e| refuse(e))
}

pub fn edit_mode() -> bool {
    MENU_STATE.service_ui.box_shown.get() == 1
}

pub fn menu_item() -> u32 {
    LOOP_FLAGS.item.get()
}

pub fn blink_on() -> bool {
    DRAW_WORK.blink.get() == 1
}

/// Menu items of the service area that have a flag: 0 .. 19. Entry 20 of both flag tables is
/// the -1 that ends them.
pub const ITEM_COUNT: u32 = 20;

/// The highlighted menu item, if it has a flag.
pub fn flagged_item() -> Result<u32, BadInput> {
    let item = menu_item();
    if item < ITEM_COUNT {
        Ok(item)
    } else {
        Err(BadInput("the highlighted menu item has no repair flag"))
    }
}

pub fn set_repair_flag(item: u32, value: u32) {
    crate::session::write_on_in(
        &MENU_LISTS,
        &MENU_LISTS.inspection_flags_a,
        item as usize,
        value,
        "a repair flag",
    )
    .unwrap_or_else(|e| refuse(e));
}

/// The flag table, with its end mark.
pub fn flags() -> rc2k_core::service::repairs::Table {
    read_all(&MENU_LISTS.inspection_flags_a)
}

pub fn store_flags(t: &rc2k_core::service::repairs::Table) {
    write_all(&MENU_LISTS.inspection_flags_a, *t);
}

/// The mark table, with its end mark.
pub fn marks() -> rc2k_core::service::repairs::Table {
    read_all(&MENU_LISTS.inspection_flags_b)
}

pub fn store_marks(t: &rc2k_core::service::repairs::Table) {
    write_all(&MENU_LISTS.inspection_flags_b, *t);
}

/// An item's repair time, through its pointer (None: null).
pub fn time_of(item: usize) -> Option<i32> {
    let key: u32 = MENU_LISTS
        .repair_time_ptrs
        .get(item)
        .map(super::super::session::Dword::get)
        .unwrap_or_else(|| refuse(BadInput("an item past the repair times' pointers")));
    crate::menu::lists_store::repair_time_at(key)
        .unwrap_or_else(|e| refuse(e))
        .map(|time| time.get() as i32)
}

/// The time of every item whose flag is not 0, as the original adds it up. Converted:
/// `rc2k_core::service::repairs::chosen_time`.
pub fn chosen_repairs_time() -> Result<i32, BadInput> {
    Ok(rc2k_core::service::repairs::chosen_time(&flags(), time_of)?)
}

pub fn leave_edit_mode() {
    MENU_STATE.service_ui.box_shown.set(0u32);
}

/// True if menu item `item` has a repair time (a non-null pointer in the table).
pub fn has_repair_time(item: u32) -> bool {
    item < ITEM_COUNT && MENU_LISTS.repair_time_ptrs[item as usize].get() != 0
}

/// True if the second flag table has its end mark where it belongs.
pub fn marks_ended() -> bool {
    rc2k_core::service::repairs::end_of(&marks()).is_some()
}
