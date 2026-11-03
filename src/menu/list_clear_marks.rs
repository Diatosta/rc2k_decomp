//! `menu::list_clear_marks` (0x414AC9): the marks (bits 6 and 7) of a list menu's items
//! cleared: each of the `[edi+1]` items (16 bytes from +0x18) its first byte `and 0x3F`,
//! and for an item of kind 2 (low nibble) its `[+1]` choices (2 bytes each from
//! `[+0xC] + 1`) too.
//!
//! In:  edi the list
//! Out: every register kept (`pusha` / `popa`); no flags read
//!
//! A list of no items goes round 2^32 times, and a kind 2 of no choices 256 times, as the original
//! (the callers' lists have items). Callers: `race::mark_viewer_items`, `sub_4144CF` (three).

use crate::menu::list_handle::ListMenu;
use rc2k_core::menu::list_menu::{has_choices, rounds, unmarked};

/// The Rust version.
pub fn list_clear_marks(list: ListMenu) {
    // Converted: `rc2k_core::menu::list_menu` (`unmarked`, `has_choices`, `rounds`).
    // Its count is a byte, but the original counts it down as a dword: 0 goes round
    // 2^32 times.
    let mut count = list.count() as u32;
    let mut n = 0;
    loop {
        let item = list.item(n);
        let kind = unmarked(item.first());
        item.set_first(kind);
        if has_choices(kind) {
            let choices = item.choices();
            for c in 0..rounds(item.count()) as u32 {
                choices.set_marks(c, unmarked(choices.marks(c)));
            }
        }
        n = n.wrapping_add(1);
        count = count.wrapping_sub(1);
        if count == 0 {
            break;
        }
    }
}
