//! The in-game options menu's (`menu::in_game_options_loop`, `off_750724`) items 3 .. 6,
//! hidden and shown by `menu::hide_in_game_options` and `menu::show_in_game_options`
//! (IDA, 2026-10-15).

use crate::framework::{game_addr, rva};
use crate::menu::disable_item::{CHOOSABLE, NOT_CHOOSABLE};
use crate::menu::item_at::{ITEM_LEN, MENU_HEAD_LEN};

pub const MENU_RVA: usize = rva(0x750724); // off_750724
/// An item's +0x1C: nothing behind its text, and what these items have when shown.
const NOTHING_BEHIND: u32 = 0xFFFF_FFA9;
const SHOWN_BEHIND: u32 = 0xFFFF_FFA7;
/// Items 3 .. 6 (counted from 0)'s strings when shown.
const FIRST_STRING: u32 = 0x171;

fn item(n: u32) -> u32 {
    game_addr(MENU_RVA) as u32 + MENU_HEAD_LEN + n * ITEM_LEN
}

unsafe fn set(n: u32, string: u32, choosable: u32, behind: u32) {
    let at = item(n);
    unsafe {
        ((at + 0x14) as *mut u32).write_unaligned(string);
        ((at + 0x48) as *mut u32).write_unaligned(choosable);
        ((at + 0x1C) as *mut u32).write_unaligned(behind);
    }
}

/// `menu::hide_in_game_options`: eax 1, 2, 3: items 3 + eax .. 6 hidden (no string, passed
/// over, nothing behind); anything else nothing.
pub fn hide_in_game_options(from: u32) {
    if (1..=3).contains(&from) {
        for n in 3 + from..=6 {
            unsafe { set(n, 0, NOT_CHOOSABLE, NOTHING_BEHIND) };
        }
    }
}

/// `menu::show_in_game_options`: items 3 .. 6 with their strings 0x171 .. 0x174, choosable.
pub fn show_in_game_options() {
    for n in 3..=6 {
        unsafe { set(n, FIRST_STRING + n - 3, CHOOSABLE, SHOWN_BEHIND) };
    }
}

/// Items 3 .. 6, the bytes the two write.
pub fn items_region() -> crate::service::harness::Region {
    (item(3), 4 * ITEM_LEN as usize)
}
