//! Game variables the service area's functions share.

use crate::errors::BadInput;
use crate::framework::{game_addr, game_read, game_write, rva};

pub const EDIT_MODE_RVA: usize = rva(0x75AA9C); // dword_75AA9C: 1 while an item is being edited
pub const EDIT_PAGE_RVA: usize = rva(0x75AAA0); // dword_75AAA0: which one, 1 .. 6
pub const MENU_ITEM_RVA: usize = rva(0x755525); // dword_755525: the highlighted menu item
pub const BLINK_RVA: usize = rva(0x7559A4); // dword_7559A4: 1 in the visible half of a blink
pub const REPAIR_FLAGS_RVA: usize = rva(0x759B6E); // dword_759B6E[item]: 1 = chosen; ends with -1
pub const REPAIR_MARKS_RVA: usize = rva(0x759BC2); // dword_759BC2[item]: a second flag per item
pub const REPAIR_TIME_PTRS_RVA: usize = rva(0x759C16); // unk_759C16[item]: pointer to its time
pub const STATUS_RVA: usize = rva(0x75AAA4); // dword_75AAA4: result of the last attempt
pub const TIME_BUDGET_RVA: usize = rva(0x75593C); // dword_75593C
pub const REPAIR_TOTAL_RVA: usize = rva(0x75AC8C); // dword_75AC8C: time of the chosen repairs
pub const PLAYER_INDEX_RVA: usize = rva(0x75598C); // dword_75598C: the current player
pub const PLAYERS_RVA: usize = rva(0x75647C); // g_players[player index]

/// The service time that is free: `budget - total` has to stay at or above this.
pub const FREE_SERVICE_TIME: i32 = 900;

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

/// Menu items of the service area that have a flag: 0 .. 19. Entry 20 of both flag tables is
/// the -1 that ends them.
pub const ITEM_COUNT: u32 = 20;

/// Bytes of a flag table with its terminator.
pub const FLAG_TABLE_LEN: usize = (ITEM_COUNT as usize + 1) * 4;

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

/// Address of the second flag of menu item `item`.
pub fn repair_mark_addr(item: u32) -> u32 {
    (game_addr(REPAIR_MARKS_RVA) as u32).wrapping_add(item.wrapping_mul(4))
}

/// The time of every item whose flag is not 0, as the original adds it up: it looks at entry
/// 0 without checking it for the end mark, and reads each time through a pointer. Refuses
/// what makes the original read through null (items 0, 1, 0x10 and 0x11 have no time) or run
/// past the table.
pub fn chosen_repairs_time() -> Result<i32, BadInput> {
    let mut sum = 0i32;
    let mut item = 0u32;
    loop {
        if repair_flag(item) != 0 {
            let time = unsafe {
                ((game_addr(REPAIR_TIME_PTRS_RVA) as *const u32).add(item as usize))
                    .read_unaligned()
            };
            if time == 0 {
                return Err(BadInput(
                    "a chosen item has no repair time: the original reads through null",
                ));
            }
            sum = sum.wrapping_add(unsafe { (time as *const i32).read_unaligned() });
        }
        item += 1;
        if item > ITEM_COUNT {
            return Err(BadInput("the repair flag table has lost its end mark"));
        }
        if repair_flag(item) == u32::MAX {
            return Ok(sum);
        }
    }
}

pub fn leave_edit_mode() {
    unsafe { game_write(EDIT_MODE_RVA, 0u32) }
}

/// True if menu item `item` has a repair time (a non-null pointer in the table).
pub fn has_repair_time(item: u32) -> bool {
    item < ITEM_COUNT
        && unsafe {
            ((game_addr(REPAIR_TIME_PTRS_RVA) as *const u32).add(item as usize)).read_unaligned()
        } != 0
}

/// True if the second flag table has its end mark where it belongs.
pub fn marks_ended() -> bool {
    (1..=ITEM_COUNT)
        .any(|item| unsafe { (repair_mark_addr(item) as *const u32).read_unaligned() } == u32::MAX)
}
