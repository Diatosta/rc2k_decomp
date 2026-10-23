//! `menu::flip_gearing` (0x469916): `dword_7554F0` flipped (`xor 1`) and its text into
//! `byte_A8BF54`: string 0x21F, or 0x221 when it is then 0, and a NUL.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`; the original loads eax just before the
//!      `popa`)
//!
//! Unlike `menu::next_gearing` it does not touch the player's record. Reached from a menu table
//! (0x74BA9C, a left/right handler of an item of the car choice). Hook removed 2026-10-22
//! (typed tables): the menus and draw lists run its `entry` (`menu::table_calls`); the
//! registers above are the original's, for reference.

use crate::framework::{game_addr, game_read, game_write};
use crate::screens::gearing_text::{KEPT_RVA, TEXT_RVA};
use crate::strings::append_string_by_id::push_string;

/// The Rust version (see the header).
pub unsafe fn flip_gearing() {
    unsafe {
        // Converted: `rc2k_core::menu::steps` (`flip`, `gearing_switch_text`).
        use rc2k_core::menu::steps::{flip, gearing_switch_text};
        let kept = flip(game_read::<u32>(KEPT_RVA));
        game_write(KEPT_RVA, kept);
        let mut text = Vec::new();
        push_string(&mut text, gearing_switch_text(kept));
        text.push(0);
        std::ptr::copy_nonoverlapping(text.as_ptr(), game_addr(TEXT_RVA) as *mut u8, text.len());
    }
}
