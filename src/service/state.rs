//! Game variables the service area's functions share.

use crate::framework::{game_addr, game_read, rva};

pub const EDIT_MODE_RVA: usize = rva(0x75AA9C); // dword_75AA9C: 1 while an item is being edited
pub const MENU_ITEM_RVA: usize = rva(0x755525); // dword_755525: the highlighted menu item
pub const BLINK_RVA: usize = rva(0x7559A4); // dword_7559A4: 1 in the visible half of a blink
pub const REPAIR_FLAGS_RVA: usize = rva(0x759B6E); // dword_759B6E[item]
pub const TIME_BUDGET_RVA: usize = rva(0x75593C); // dword_75593C
pub const REPAIR_TOTAL_RVA: usize = rva(0x75AC8C); // dword_75AC8C: time of the chosen repairs
pub const CAR_INDEX_RVA: usize = rva(0x75598C); // dword_75598C
pub const CARS_RVA: usize = rva(0x75647C); // off_75647C[car index]: car objects

/// The service time that is free: `budget - total` has to stay at or above this.
pub const FREE_SERVICE_TIME: i32 = 900;

/// The current car object, or None if the index or the pointer is not usable.
pub fn current_car() -> Option<u32> {
    unsafe {
        let index: u32 = game_read(CAR_INDEX_RVA);
        if index >= 64 {
            return None;
        }
        let car = ((game_addr(CARS_RVA) as *const u32).add(index as usize)).read_unaligned();
        (car != 0).then_some(car)
    }
}

/// Address of the flag of menu item `item` in the repair flag table.
pub fn repair_flag_addr(item: u32) -> u32 {
    (game_addr(REPAIR_FLAGS_RVA) as u32).wrapping_add(item.wrapping_mul(4))
}

pub fn repair_flag(item: u32) -> u32 {
    unsafe { (repair_flag_addr(item) as *const u32).read_unaligned() }
}

pub fn edit_mode() -> bool {
    unsafe { game_read::<u32>(EDIT_MODE_RVA) == 1 }
}

pub fn menu_item() -> u32 {
    unsafe { game_read(MENU_ITEM_RVA) }
}

pub fn blink_on() -> bool {
    unsafe { game_read::<u32>(BLINK_RVA) == 1 }
}
