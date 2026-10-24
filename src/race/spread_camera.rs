//! `race::spread_camera` (0x41E6BD): a block's camera byte changed (ebx, the old value in
//! dh): the new value (`[ebx]`) also into the next blocks' same byte (8 bytes on each)
//! while they held the old one, for the blocks after the one shown
//! (`dword_5BECD0 - dword_5C2530` less one at most).
//!
//! In:  ebx the byte, dh its old value
//! Out: dl the new value, eax the blocks gone over (from 1), ecx what was left of the
//!      count; no flags read
//!
//! Callers: `race::viewer_action` (four), `race::next_camera`, `race::previous_camera`,
//! with the block shown not the last. The count is not checked: 0 (or less) is limited
//! only by the bytes met; refused when the walk would pass the 0x708 blocks of the
//! table, before anything is written. Shadow mode compares. Self-test: a table of the
//! test's, runs of the old value of every length, the count reached or not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::race::viewer::{BLOCK, BLOCKS, get};

/// The Rust version: `byte` the changed camera byte's address, `old` its old value. What
/// the original leaves (dl, eax, ecx) its callers do not read: `viewer_action` and the
/// camera steps end with it, and `viewer_action` keeps every register.
pub unsafe fn spread_camera(byte: u32, old: u8) -> Result<(), Refused> {
    let at = |n: u32| (byte.wrapping_add(n.wrapping_mul(8))) as *mut u8;
    let dl = unsafe { (byte as *const u8).read() };
    let count = get(BLOCKS).wrapping_sub(get(BLOCK));
    // Converted: `rc2k_core::race::viewer::spread` (how far it goes, first).
    let s = rc2k_core::race::viewer::spread(count, old, |n| unsafe { at(n).read() })?;
    for n in 1..=s.taken {
        unsafe { at(n).write(dl) };
    }
    Ok(())
}
