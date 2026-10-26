//! `net::draw_centred_item` (0x416B53): an item's string drawn (`text::draw_string_by_id`)
//! at x ecx, y ebp, in `unk_51E744`, or `stru_51E684` when it is the highlighted item and
//! edited or in the blink's lit half (as `net::draw_button`); ebp moves down by the font's
//! line height (+8).
//!
//! In:  eax = the string, ecx = x, esi = the item, ebp = y
//! Out: ebx = the font, edx = the y, ebp lower; eax .. edx as the drawing leaves them; no
//!      `pusha`
//! Caller: `net::settings_frame` (its last item).

use crate::errors::Refused;
use crate::net::draw_button::item_font;
use crate::text::draw_text::GameCanvas;

const PLAIN: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::CENTRED_PLAIN;
const LIT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::CENTRED_LIT;

/// The Rust version: the string `string` for item `item` at (`x`, `y`) in the item's font;
/// returns the next y (y plus the font's line height; the original's ebp). The registers it
/// leaves (ebx the font, edx the y, eax .. edx as the drawing leaves them) are read by no
/// caller (`net::settings_frame` only).
pub fn draw_centred_item(string: u32, item: u32, x: u32, y: u32) -> Result<u32, Refused> {
    // Converted: `rc2k_core::net::screens::item_lit` (in `item_font`).
    let font = item_font(item, PLAIN, LIT);
    let next = y.wrapping_add(font.object().line_height.get());
    unsafe {
        crate::text::draw_string_by_id::draw_string_by_id_at(
            string,
            font.as_ptr(),
            x as i32,
            y as i32,
            &mut GameCanvas,
        )
    }?;
    Ok(next)
}
