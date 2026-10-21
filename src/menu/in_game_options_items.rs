//! The in-game options menu's (`menu::in_game_options_loop`, `off_750724`) items 3 .. 6,
//! hidden and shown by `menu::hide_in_game_options` and `menu::show_in_game_options`
//! (IDA, 2026-10-15).

use crate::framework::{game_addr, rva};
use crate::menu::item_at::{ITEM_LEN, MENU_HEAD_LEN};
use rc2k_core::menu::items::{
    HIDDEN, IN_GAME_OPTIONS, Look, hidden_in_game_options, shown_in_game_option,
};

pub const MENU_RVA: usize = rva(0x750724); // off_750724

pub fn item(n: u32) -> u32 {
    game_addr(MENU_RVA) as u32 + MENU_HEAD_LEN + n * ITEM_LEN
}

/// Item `n` given its look: +0x14, +0x48, +0x1C, in the original's order.
pub unsafe fn set(n: u32, look: Look) {
    let at = item(n);
    unsafe {
        ((at + 0x14) as *mut u32).write_unaligned(look.string);
        ((at + 0x48) as *mut u32).write_unaligned(look.state);
        ((at + 0x1C) as *mut u32).write_unaligned(look.behind);
    }
}

// Converted: `rc2k_core::menu::items` (`hidden_in_game_options`, `shown_in_game_option`).

/// `menu::hide_in_game_options`: eax 1, 2, 3: items 3 + eax .. 6 hidden (no string, passed
/// over, nothing behind); anything else nothing.
pub fn hide_in_game_options(from: u32) {
    for n in hidden_in_game_options(from) {
        unsafe { set(n, HIDDEN) };
    }
}

/// `menu::show_in_game_options`: items 3 .. 6 with their strings 0x171 .. 0x174, choosable.
pub fn show_in_game_options() {
    for n in IN_GAME_OPTIONS {
        unsafe { set(n, shown_in_game_option(n)) };
    }
}

/// Items 3 .. 6, the bytes the two write.
pub fn items_region() -> crate::service::harness::Region {
    (item(IN_GAME_OPTIONS.start), 4 * ITEM_LEN as usize)
}
