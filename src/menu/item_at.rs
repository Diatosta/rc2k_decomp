//! `menu::item_at` (0x471CEF): the address of item number ebx of the current menu.
//!
//! In:  ebx = item number, from 1
//! Out: edi = the item, eax = ebx; everything else preserved
//! The current menu is `dword_75558D`; its items are 0x5C bytes each after a 4-byte head, so
//! item n is at menu + 4 + (n - 1) * 0x5C. The original counts up to ebx one item at a time
//! and never looks for the end of the menu. 14 callers.
//! Refuses item number 0, for which the original counts through all 32 bits.

use crate::errors::BadInput;
use crate::framework::{game_read, rva};

// ---- game data ----

pub const CURRENT_MENU_RVA: usize = rva(0x75558D); // dword_75558D
pub use rc2k_core::menu::items::{ITEM_LEN, MENU_HEAD_LEN};

// ---- Rust version ----

pub fn item_at(number: u32) -> Result<u32, BadInput> {
    // Converted: `rc2k_core::menu::items::item_at`.
    Ok(rc2k_core::menu::items::item_at(number, || unsafe {
        game_read(CURRENT_MENU_RVA)
    })?)
}
