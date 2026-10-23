//! `menu::list_item_unusable` (0x414C1B): whether the highlighted item of a list menu
//! (`dword_51D407`, 16 bytes from +0x18) cannot be chosen: its first byte's bits 6 and 7
//! exactly 0x40 or 0x80, or its kind (low nibble) 5.
//!
//! In:  edi the list
//! Out: carry set when it cannot be chosen; every register kept (eax pushed)
//!
//! Callers: `race::replay_viewer` (three), `sub_4144CF` (three), reading carry. Shadow mode
//! compares. Self-test: every mark and kind of the item against the original.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::viewer::{CHOICE, get, regs_entry};

/// The Rust version: carry.
pub unsafe fn list_item_unusable(list: u32) -> bool {
    let at = get(CHOICE)
        .wrapping_mul(0x10)
        .wrapping_add(list)
        .wrapping_add(0x18);
    // Converted: `rc2k_core::menu::list_menu::unusable`.
    rc2k_core::menu::list_menu::unusable(unsafe { (at as *const u8).read() })
}

fn rust(r: &Regs) -> Result<Regs, Refused> {
    let mut left = *r;
    left.set_carry(unsafe { list_item_unusable(r.edi) });
    Ok(left)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &rust)
}
