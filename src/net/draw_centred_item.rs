//! `net::draw_centred_item` (0x416B53): an item's string drawn (`text::draw_string_by_id`)
//! at x ecx, y ebp, in `unk_51E744`, or `stru_51E684` when it is the highlighted item and
//! edited or in the blink's lit half (as `net::draw_button`); ebp moves down by the font's
//! line height (+8).
//!
//! In:  eax = the string, ecx = x, esi = the item, ebp = y
//! Out: ebx = the font, edx = the y, ebp lower; eax .. edx as the drawing leaves them; no
//!      `pusha`
//! Caller: `net::settings_frame` (its last item). Its call through `part`. Shadow mode runs
//! the original alone (it draws). Self-test: a copy whose call goes to a stand-in, against
//! the Rust version: highlighted or not, edited, blinking.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::net::draw_button::item_font;
use crate::text::draw_text::GameCanvas;
use crate::text::font::Font;

const PLAIN: usize = 0x51E744; // unk_51E744
const LIT: usize = 0x51E684; // stru_51E684

/// The Rust version: the string `string` for item `item` at (`x`, `y`) in the item's font;
/// returns the next y (y plus the font's line height; the original's ebp). The registers it
/// leaves (ebx the font, edx the y, eax .. edx as the drawing leaves them) are read by no
/// caller (`net::settings_frame` only).
pub fn draw_centred_item(string: u32, item: u32, x: u32, y: u32) -> Result<u32, Refused> {
    // Converted: `rc2k_core::net::screens::item_lit` (in `item_font`).
    let font = item_font(item, PLAIN, LIT);
    let next = y.wrapping_add(unsafe { ((font + 8) as *const u32).read_unaligned() });
    unsafe {
        crate::text::draw_string_by_id::draw_string_by_id_at(
            string,
            font as *mut Font,
            x as i32,
            y as i32,
            &mut GameCanvas,
        )
    }?;
    Ok(next)
}
