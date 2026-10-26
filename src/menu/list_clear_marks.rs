//! `menu::list_clear_marks` (0x414AC9): the marks (bits 6 and 7) of a list menu's items
//! cleared: each of the `[edi+1]` items (16 bytes from +0x18) its first byte `and 0x3F`,
//! and for an item of kind 2 (low nibble) its `[+1]` choices (2 bytes each from
//! `[+0xC] + 1`) too.
//!
//! In:  edi the list
//! Out: every register kept (`pusha` / `popa`); no flags read
//!
//! A list of no items goes round 2^32 times, and a kind 2 of no choices 256 times, as
//! the original (the callers' lists have items). Callers: `race::mark_viewer_items`,
//! `sub_4144CF` (three). Shadow mode compares. Self-test: lists of the test's (kinds 2 and
//! others, marks set) against the original.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::viewer::regs_entry;
use crate::service::harness::Region;
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

fn rust(r: &Regs) -> Result<Regs, Refused> {
    unsafe { list_clear_marks(r.edi) };
    Ok(*r)
}

/// The list's bytes and its kind 2 items' choices, as far as they go.
pub unsafe fn regions_of(list: u32) -> Vec<Region> {
    unsafe {
        let count = ((list + 1) as *const u8).read() as usize;
        let mut all = vec![(list, 0x18 + 0x10 * count)];
        for n in 0..count {
            let item = list + 0x18 + 0x10 * n as u32;
            if (item as *const u8).read() & 0xF == 2 {
                let choices = ((item + 1) as *const u8).read() as usize;
                let at = ((item + 0xC) as *const u32).read_unaligned();
                let n = if choices == 0 { 0x100 } else { choices };
                all.push((at, 2 * n));
            }
        }
        all
    }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &rust)
}
