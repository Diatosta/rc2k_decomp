//! Draws the on-screen keyboard (`rc2k_core::input::osk`) over the frame about to be shown: a
//! dark panel above or below the field being edited (never over it), one tile per key, the key
//! under the cursor on a bright tile, each key's text centred on its tile in the menus' list
//! font. On both renderers: solid rectangles go to the GPU as quads where it draws the screen,
//! else row by row on the canvas.

use crate::gfx::typefaces::fonts;
use crate::text::draw_text::{Canvas, GameCanvas, draw_text_at};
use rc2k_core::input::osk::{self, Key, Osk};

/// A key's tile, and the gap between tiles.
const CELL_W: i32 = 52;
const CELL_H: i32 = 40;
const GAP: i32 = 4;
/// The panel's border round the tiles.
const MARGIN: i32 = 12;
/// The space kept between the field and the panel.
const AWAY: i32 = 36;
/// The menus' screen.
const SCREEN_W: i32 = 800;
const SCREEN_H: i32 = 600;
/// Where the panel goes when the field's place is not known.
const DEFAULT_TOP: i32 = 300;

/// Colours (0xRRGGBB).
const PANEL: u32 = 0x0C1238;
const TILE: u32 = 0x27347A;
const TILE_COMMAND: u32 = 0x1B2560;
const TILE_LIT: u32 = 0xC8202A;

/// The panel's size.
fn panel_size() -> (i32, i32) {
    (
        osk::COLUMNS as i32 * CELL_W + 2 * MARGIN - GAP,
        osk::ROWS as i32 * CELL_H + 2 * MARGIN - GAP,
    )
}

/// The panel's top: below the field when it fits there, else above it.
fn panel_top(field_y: Option<i32>) -> i32 {
    let (_, h) = panel_size();
    let Some(y) = field_y else {
        return DEFAULT_TOP;
    };
    let below = y + AWAY;
    if below + h <= SCREEN_H - 8 {
        below
    } else {
        (y - AWAY / 2 - h).max(8)
    }
}

/// The y of the menu item being edited, if the screen is a menu's.
fn field_y(o: crate::owners::Owners<'static>) -> Option<i32> {
    let flags = &o.menus.screens.loop_flags;
    let menu = flags.menu.get().ok()?;
    let item = menu.item(&o.menus.tables.menus, flags.item.get()).ok()?;
    Some(item.fixed().y)
}

fn pixel(rgb: u32, bits: u32) -> u32 {
    if bits == 16 {
        let (r, g, b) = (rgb >> 16 & 0xFF, rgb >> 8 & 0xFF, rgb & 0xFF);
        (r >> 3) << 11 | (g >> 2) << 5 | b >> 3
    } else {
        rgb
    }
}

/// A solid rectangle (x, y, width, height) of `rgb`.
fn fill(o: crate::owners::Owners<'static>, rect: [i32; 4], rgb: u32) {
    let surface = crate::gfx::surface::surface(o.gfx);
    let value = pixel(rgb, surface.bits_per_pixel);
    if surface.bits_per_pixel == 32
        && crate::gfx::draw2d::quads::note_solid(&[rect], value.to_le_bytes(), surface.pitch)
    {
        return;
    }
    let entry = [crate::memory::session::PlainDword::new(value)];
    let palette = crate::gfx::rect::Palette::Of(&entry);
    let [x, y, w, h] = rect;
    for row in y..y + h {
        crate::gfx::rect::draw_hline(x, row, w, 0, palette, o.gfx);
    }
}

/// The text a key is drawn with, ended by its NUL.
fn text_of(osk: &Osk, key: Key) -> Vec<u8> {
    let mut text = match key {
        Key::Char(c) => vec![c],
        key => osk.label(key).to_vec(),
    };
    text.push(0);
    text
}

/// A text's width in a font, measured without drawing.
fn width_of(font: crate::gfx::typefaces::FontId, text: &[u8], canvas: &mut GameCanvas) -> i32 {
    let mut copy = font.object(canvas.typefaces()).get();
    copy.mode = rc2k_core::strings::text::MODE_MEASURE;
    crate::text::measure_text::measure_text(&mut copy, text, 0, canvas)
        .map(|m| m.width as i32)
        .unwrap_or(0)
}

/// The keyboard drawn if one is shown. The surface is locked for the drawing when it is not.
pub fn draw(o: crate::owners::Owners<'static>) {
    let Some(osk) = crate::input::typed_text::osk() else {
        return;
    };
    let locked_here = o.gfx.locked.surface.get() != 1;
    if locked_here && crate::platform::lock_surface(o.gfx, o.system) < 1 {
        return;
    }
    let (w, h) = panel_size();
    let left = (SCREEN_W - w) / 2;
    let top = panel_top(field_y(o));
    fill(o, [left, top, w, h], PANEL);
    let mut canvas = GameCanvas::new(o.gfx);
    let font = fonts::LIST;
    'rows: for row in 0..osk::ROWS {
        let keys = Osk::width(row) as i32;
        // The command row's keys are wider: the row spans the same width.
        let span = (w - 2 * MARGIN + GAP) / keys;
        for col in 0..keys {
            let key = osk.key_at(row, col as usize);
            let x = left + MARGIN + col * span;
            let y = top + MARGIN + row as i32 * CELL_H;
            let lit = (row, col as usize) == (osk.row, osk.col);
            let colour = match (lit, key) {
                (true, _) => TILE_LIT,
                (false, Key::Char(_)) => TILE,
                (false, _) => TILE_COMMAND,
            };
            fill(o, [x, y, span - GAP, CELL_H - GAP], colour);
            let text = text_of(&osk, key);
            let tx = x + (span - GAP - width_of(font, &text, &mut canvas)) / 2;
            // The font's own top offset puts its letters a little below the y it is given.
            let ty = y + (CELL_H - GAP) / 2 - 12;
            if let Err(why) = draw_text_at(font, &text[..], tx, ty, &mut canvas) {
                crate::logfile::program_log::log(&format!("osk: not drawn: {why:?}"));
                break 'rows;
            }
        }
    }
    if locked_here {
        crate::platform::unlock_surface(o.gfx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_panel_keeps_off_the_field() {
        let (_, h) = panel_size();
        // a field near the top: below it
        assert_eq!(panel_top(Some(220)), 220 + AWAY);
        // a field low down: above it, clear of it
        let top = panel_top(Some(450));
        assert!(top + h <= 450);
        assert_eq!(panel_top(None), DEFAULT_TOP);
    }
}
