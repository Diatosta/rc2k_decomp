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

use crate::errors::BadInput;
use crate::pointers::GamePtr;
use rc2k_core::menu::list_menu::{has_choices, rounds, unmarked};

/// The Rust version.
pub fn list_clear_marks(list: u32) -> Result<(), BadInput> {
    // Converted: `rc2k_core::menu::list_menu` (`unmarked`, `has_choices`, `rounds`).
    // Its count is a byte, but the original counts it down as a dword: 0 goes round
    // 2^32 times.
    let mut count = GamePtr::new(list + 1).read::<u8>("a list menu\'s count")? as u32;
    let mut item = GamePtr::new(list + 0x18);
    loop {
        let kind = unmarked(item.read::<u8>("a list item")?);
        item.write::<u8>(kind, "a list item")?;
        if has_choices(kind) {
            let n = rounds(item.offset(1).read::<u8>("a list item")?);
            let choices = GamePtr::new(item.offset(0xC).read::<u32>("a list item")? + 1);
            for c in 0..n as u32 {
                let choice = choices.offset(2 * c);
                choice.write::<u8>(
                    unmarked(choice.read::<u8>("a list choice")?),
                    "a list choice",
                )?;
            }
        }
        item = item.offset(0x10);
        count = count.wrapping_sub(1);
        if count == 0 {
            break;
        }
    }
    Ok(())
}
