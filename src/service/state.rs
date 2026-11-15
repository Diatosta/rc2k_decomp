//! Game variables the service area's functions share.

use crate::errors::BadInput;
use crate::menu::owner::Menus;
use crate::menu::owner::Screens;
use crate::service::owner::Service;
use crate::session::{PlainDword, read_all, write_all};
use crate::system::checked_reads::refuse;

/// The current player (`Player`), or None if the index is past the four.
pub fn current_player(menus: &Menus) -> Option<crate::menu::standings::PlayerCell<'_>> {
    crate::menu::player::player(menus, menus.screens.event.current_player.get())
}

/// The flag of menu item `item` in the repair flag table. The items are the workshop's and the
/// markers' constants and the highlighted item held below `ITEM_COUNT`: 0 ..= 20, the table's
/// 20 flags and its end mark.
pub fn repair_flag(service: &Service, item: u32) -> u32 {
    repair_flag_in(service, item)
}

/// `repair_flag` for a caller that holds the tables group.
pub fn repair_flag_in(service: &Service, item: u32) -> u32 {
    service
        .repair_flags
        .get(item as usize)
        .unwrap_or_else(|| refuse(BadInput("a repair item past the flag table")))
        .get()
}

pub fn edit_mode(menus: &Menus) -> bool {
    menus.screens.state.service_ui.box_shown.is_set()
}

pub fn menu_item(menus: &Menus) -> u32 {
    menus.screens.loop_flags.item.get()
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

pub fn set_repair_flag(service: &Service, item: u32, value: u32) {
    service
        .repair_flags
        .get(item as usize)
        .unwrap_or_else(|| refuse(BadInput("a repair item past the flag table")))
        .set(value);
}

/// The flag table, with its end mark.
pub fn flags(service: &Service) -> rc2k_core::service::repairs::Table {
    read_all(&service.repair_flags)
}

pub fn store_flags(service: &Service, t: &rc2k_core::service::repairs::Table) {
    write_all(&service.repair_flags, *t);
}

/// The mark table, with its end mark.
pub fn marks(service: &Service) -> rc2k_core::service::repairs::Table {
    read_all(&service.repair_marks)
}

pub fn store_marks(service: &Service, t: &rc2k_core::service::repairs::Table) {
    write_all(&service.repair_marks, *t);
}

/// An item's repair time, through its pointer (None: null).
pub fn time_of(service: &Service, item: usize) -> Option<i32> {
    let key: u32 = service
        .repair_time_handles
        .get(item)
        .map(crate::session::DwordLike::get)
        .unwrap_or_else(|| refuse(BadInput("an item past the repair times' pointers")));
    repair_time_at(service, key)
        .unwrap_or_else(|e| refuse(e))
        .map(|time| time.get() as i32)
}

/// An item's repair time cell, by its handle in `Service::repair_time_handles` (None: none).
fn repair_time_at(service: &Service, key: u32) -> Result<Option<&PlainDword>, BadInput> {
    match key {
        0 => Ok(None),
        _ => service
            .repair_times
            .get(key.wrapping_sub(1) as usize)
            .map(Some)
            .ok_or(BadInput("a repair time's handle off the times")),
    }
}

/// The time of every item whose flag is not 0.
pub fn chosen_repairs_time(service: &Service) -> Result<i32, BadInput> {
    Ok(rc2k_core::service::repairs::chosen_time(
        &flags(service),
        |item| time_of(service, item),
    )?)
}

pub fn leave_edit_mode(screens: &Screens) {
    screens.state.service_ui.box_shown.set(false);
}

/// True if menu item `item` has a repair time (a non-null pointer in the table).
pub fn has_repair_time(service: &Service, item: u32) -> bool {
    item < ITEM_COUNT && service.repair_time_handles[item as usize].is_set()
}

/// True if the second flag table has its end mark where it belongs.
pub fn marks_ended(service: &Service) -> bool {
    rc2k_core::service::repairs::end_of(&marks(service)).is_some()
}
