//! Game variables the service area's functions share.

use crate::errors::BadInput;
use crate::menu::draw_work_store::DRAW_WORK;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::lists_store::MENU_LISTS;
use crate::menu::loop_store::LOOP_FLAGS;
use crate::menu::players_store::PLAYER_TABLE;
use crate::menu::state_store::MENU_STATE;
use crate::pointers::GamePtr;
use crate::session::{read_all, write_all};
use crate::system::checked_reads::refuse;

/// The current player (`Player`), or None if the index or the pointer is not usable.
pub fn current_player() -> Option<u32> {
    let index: u32 = EVENT_STATE.current_player.get();
    if index >= 64 {
        return None;
    }
    let player = GamePtr::new(PLAYER_TABLE.pointers[0].addr())
        .offset(4 * index)
        .read::<u32>("the players' table")
        .unwrap_or_else(|e| refuse(e));
    (player != 0).then_some(player)
}

/// Address of the flag of menu item `item` in the repair flag table.
pub fn repair_flag_addr(item: u32) -> u32 {
    (MENU_LISTS.inspection_flags_a[0].addr()).wrapping_add(item.wrapping_mul(4))
}

pub fn repair_flag(item: u32) -> u32 {
    GamePtr::new(repair_flag_addr(item))
        .read::<u32>("a repair flag")
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
    GamePtr::new(repair_flag_addr(item))
        .write(value, "a repair flag")
        .unwrap_or_else(|e| refuse(e))
}

/// The flag table, with its end mark.
pub fn flags() -> rc2k_core::service::repairs::Table {
    read_all(&MENU_LISTS.inspection_flags_a)
}

pub fn store_flags(t: &rc2k_core::service::repairs::Table) {
    write_all(&MENU_LISTS.inspection_flags_a, *t)
}

/// The mark table, with its end mark.
pub fn marks() -> rc2k_core::service::repairs::Table {
    read_all(&MENU_LISTS.inspection_flags_b)
}

pub fn store_marks(t: &rc2k_core::service::repairs::Table) {
    write_all(&MENU_LISTS.inspection_flags_b, *t)
}

/// An item's repair time, through its pointer (None: null).
pub fn time_of(item: usize) -> Option<i32> {
    let at = GamePtr::new(MENU_LISTS.repair_time_ptrs[0].addr())
        .offset(4 * item as u32)
        .read::<u32>("the repair times' pointers")
        .unwrap_or_else(|e| refuse(e));
    (at != 0).then(|| {
        GamePtr::new(at)
            .read::<i32>("a repair time")
            .unwrap_or_else(|e| refuse(e))
    })
}

/// The time of every item whose flag is not 0, as the original adds it up. Converted:
/// `rc2k_core::service::repairs::chosen_time`.
pub fn chosen_repairs_time() -> Result<i32, BadInput> {
    Ok(rc2k_core::service::repairs::chosen_time(&flags(), time_of)?)
}

pub fn leave_edit_mode() {
    MENU_STATE.service_ui.box_shown.set(0u32)
}

/// True if menu item `item` has a repair time (a non-null pointer in the table).
pub fn has_repair_time(item: u32) -> bool {
    item < ITEM_COUNT
        && GamePtr::new(MENU_LISTS.repair_time_ptrs[0].addr())
            .offset(4 * item)
            .read::<u32>("the repair times' pointers")
            .unwrap_or_else(|e| refuse(e))
            != 0
}

/// True if the second flag table has its end mark where it belongs.
pub fn marks_ended() -> bool {
    rc2k_core::service::repairs::end_of(&marks()).is_some()
}
