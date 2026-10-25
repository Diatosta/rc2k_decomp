//! Game variables the service area's functions share.

use crate::errors::BadInput;
use crate::framework::{game_addr, game_read, rva};
use crate::menu::lists_store::MENU_LISTS;
use crate::menu::state_store::MENU_STATE;
use crate::session::{read_all, write_all};

pub const MENU_ITEM_RVA: usize = rva(0x755525); // dword_755525: the highlighted menu item
pub const BLINK_RVA: usize = rva(0x7559A4); // dword_7559A4: 1 in the visible half of a blink
pub const TIME_BUDGET_RVA: usize = rva(0x75593C); // dword_75593C
pub const PLAYER_INDEX_RVA: usize = rva(0x75598C); // dword_75598C: the current player
pub const PLAYERS_RVA: usize = rva(0x75647C); // g_players[player index]

/// The current player (`Player`), or None if the index or the pointer is not usable.
pub fn current_player() -> Option<u32> {
    unsafe {
        let index: u32 = game_read(PLAYER_INDEX_RVA);
        if index >= 64 {
            return None;
        }
        let player = ((game_addr(PLAYERS_RVA) as *const u32).add(index as usize)).read_unaligned();
        (player != 0).then_some(player)
    }
}

/// Address of the flag of menu item `item` in the repair flag table.
pub fn repair_flag_addr(item: u32) -> u32 {
    (MENU_LISTS.inspection_flags_a[0].addr() as u32).wrapping_add(item.wrapping_mul(4))
}

pub fn repair_flag(item: u32) -> u32 {
    unsafe { (repair_flag_addr(item) as *const u32).read_unaligned() }
}

pub fn edit_mode() -> bool {
    MENU_STATE.service_ui.box_shown.get() == 1
}

pub fn menu_item() -> u32 {
    unsafe { game_read(MENU_ITEM_RVA) }
}

pub fn blink_on() -> bool {
    unsafe { game_read::<u32>(BLINK_RVA) == 1 }
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
    unsafe { (repair_flag_addr(item) as *mut u32).write_unaligned(value) }
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
    let at = unsafe {
        ((MENU_LISTS.repair_time_ptrs[0].addr() as *const u32).add(item)).read_unaligned()
    };
    (at != 0).then(|| unsafe { (at as *const i32).read_unaligned() })
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
        && unsafe {
            ((MENU_LISTS.repair_time_ptrs[0].addr() as *const u32).add(item as usize))
                .read_unaligned()
        } != 0
}

/// True if the second flag table has its end mark where it belongs.
pub fn marks_ended() -> bool {
    rc2k_core::service::repairs::end_of(&marks()).is_some()
}
