//! `menu::stage_range_caption` (0x4A1FBB): "first-last" into `g_captionText`: the first
//! (`dword_7556B8`) and it plus 9 (a reading: the stages shown on a page).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the event pages' code (`sub_4A1F7A`, ..). `strings::append_u32`'s Rust
//! version; the minus and the NUL written in place.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::framework::{game_addr, game_read, rva};

const FIRST_RVA: usize = rva(0x7556B8); // dword_7556B8
const CAPTION_RVA: usize = rva(0xA8BE54); // g_captionText

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn stage_range_caption() {
    let first: u32 = unsafe { game_read(FIRST_RVA) };
    // Converted: `rc2k_core::menu::event::stage_range`.
    let mut text = rc2k_core::menu::event::stage_range(first);
    text.push(0);
    unsafe {
        std::ptr::copy_nonoverlapping(text.as_ptr(), game_addr(CAPTION_RVA) as *mut u8, text.len())
    };
}
