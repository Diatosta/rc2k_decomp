//! `net::draw_centred_item` (0x416B53): an item's string drawn (`text::draw_string_by_id`)
//! at x, y, in `typefaces::CENTRED_PLAIN`, or `typefaces::CENTRED_LIT` when it is the highlighted item and
//! edited or in the blink's lit half (as `net::draw_button`); the y moves down by the font's
//! line height (+8).
//!
//! Takes the string, x, the item and y.
//! Caller: `net::settings_frame` (its last item).

use crate::errors::Refused;
use crate::net::draw_button::item_font;
use crate::text::draw_text::GameCanvas;

const PLAIN: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::CENTRED_PLAIN;
const LIT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::CENTRED_LIT;

/// The string `string` for item `item` at (`x`, `y`) in the item's font;
/// returns the next y (y plus the font's line height).
pub fn draw_centred_item(
    o: crate::owners::Owners<'static>,
    net: &crate::net::owner::Net,
    install: &crate::install::owner::Install,
    string: u32,
    item: u32,
    x: u32,
    y: u32,
) -> Result<u32, Refused> {
    // Converted: `rc2k_core::net::screens::item_lit` (in `item_font`).
    let font = item_font(net, install, item, PLAIN, LIT);
    let next = y.wrapping_add(font.object(&o.gfx.fonts.typefaces).line_height.get());
    crate::text::draw_string_by_id::draw_string_by_id_at(
        crate::strings::append_string_by_id::StringOwners::of(o),
        string,
        font,
        x as i32,
        y as i32,
        &mut GameCanvas::new(o.gfx),
    )?;
    Ok(next)
}
