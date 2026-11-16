//! Draws an item's string centred at x, y, in the lit font when it is the highlighted item and
//! edited or in the blink's lit half (as `net::draw_button`); y moves down by the font's line
//! height.

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
    string: u32,
    item: u32,
    x: u32,
    y: u32,
) -> Result<u32, Refused> {
    // The logic is in `rc2k_core::net::screens::item_lit` (in `item_font`).
    let font = item_font(&net.settings().edit, &o.menus.list, item, PLAIN, LIT);
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
