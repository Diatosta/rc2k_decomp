//! `menu::item_at` (0x471CEF): item number ebx of the current menu (by handle since
//! 2026-10-28: `menus_store::MenuId::item`).
//!
//! In:  ebx = item number, from 1
//! Out: edi = the item, eax = ebx; everything else preserved
//! The current menu is `dword_75558D`; its items are 0x5C bytes each after a 4-byte head, so
//! item n is at menu + 4 + (n - 1) * 0x5C. The original counts up to ebx one item at a time
//! and never looks for the end of the menu. 14 callers.
//! Refuses item number 0, for which the original counts through all 32 bits.

use crate::errors::BadInput;
use crate::menu::menus_store::MenuItem;
use crate::menu::owner::Screens;

// ---- Rust version ----

/// Item `number` (from 1) of the current menu.
pub fn item_at(screens: &Screens, number: u32) -> Result<&'_ MenuItem, BadInput> {
    screens.loop_flags.menu.get()?.item(number)
}
