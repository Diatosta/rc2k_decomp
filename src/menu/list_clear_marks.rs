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

use rc2k_core::menu::list_menu::{has_choices, rounds, unmarked};

/// The Rust version.
pub unsafe fn list_clear_marks(list: u32) {
    // Converted: `rc2k_core::menu::list_menu` (`unmarked`, `has_choices`, `rounds`).
    unsafe {
        // Its count is a byte, but the original counts it down as a dword: 0 goes round
        // 2^32 times.
        let mut count = ((list + 1) as *const u8).read() as u32;
        let mut item = (list + 0x18) as *mut u8;
        loop {
            let kind = unmarked(item.read());
            item.write(kind);
            if has_choices(kind) {
                let n = rounds(item.add(1).read());
                let choices = ((item.add(0xC) as *const u32).read_unaligned() + 1) as *mut u8;
                for c in 0..n as usize {
                    let choice = choices.add(2 * c);
                    choice.write(unmarked(choice.read()));
                }
            }
            item = item.add(0x10);
            count = count.wrapping_sub(1);
            if count == 0 {
                break;
            }
        }
    }
}
