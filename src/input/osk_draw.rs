//! Draws the on-screen keyboard (`rc2k_core::input::osk`) over the frame about to be shown: a
//! dark panel above or below the field being edited (never over it), one tile per key, the key
//! under the cursor on a bright tile, each key's text centred on its tile in the menus' list
//! font. The panel and the tiles are written into the frame's pixels on every renderer: what the
//! menu left there under the panel is covered, and the button pictures blend their edges with
//! the panel's colour (as the GPU's quads they would sit under the frame's written pixels).

use crate::gfx::button_hints::width_of;
use crate::gfx::typefaces::fonts;
use crate::text::draw_text::{GameCanvas, draw_text_at};
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

/// The line at the panel's top that shows what has been typed, and the gap under it.
const ECHO_H: i32 = 32;
const ECHO_GAP: i32 = 8;
/// The echo box's colour.
const ECHO_BOX: u32 = 0x060A22;

/// The strip at the panel's foot for the hints of the controller's buttons.
const HINT_STRIP: i32 = 40;

/// The width of the keys' block.
fn keys_width() -> i32 {
    osk::COLUMNS as i32 * CELL_W - GAP
}

/// The panel's size: wide enough for the keys and, with a margin each side, for a hint line of
/// `hint_width` (0 for none).
fn panel_size(hint_width: i32) -> (i32, i32) {
    (
        keys_width().max(hint_width) + 2 * MARGIN,
        ECHO_H + ECHO_GAP + osk::ROWS as i32 * CELL_H + 2 * MARGIN - GAP + HINT_STRIP,
    )
}

/// The panel's top: below the field when it fits there, else above it.
fn panel_top(field_y: Option<i32>) -> i32 {
    let (_, h) = panel_size(0);
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
    if let Some(y) = crate::input::typed_text::field_y() {
        return Some(y);
    }
    let flags = &o.menus.screens.loop_flags;
    let menu = flags.menu.get().ok()?;
    let item = menu.item(&o.menus.tables.menus, flags.item.get()).ok()?;
    Some(item.fixed().y)
}

/// A colour (0xRRGGBB) as a pixel of a surface of `bits` bits.
pub fn pixel(rgb: u32, bits: u32) -> u32 {
    if bits == 16 {
        let (r, g, b) = (rgb >> 16 & 0xFF, rgb >> 8 & 0xFF, rgb & 0xFF);
        (r >> 3) << 11 | (g >> 2) << 5 | b >> 3
    } else {
        rgb
    }
}

/// A solid rectangle (x, y, width, height) of `rgb`.
pub fn fill(o: crate::owners::Owners<'static>, rect: [i32; 4], rgb: u32) {
    let surface = crate::gfx::surface::surface(o.gfx);
    let value = pixel(rgb, surface.bits_per_pixel);
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

/// The keyboard drawn if one is shown. The surface is locked for the drawing when it is not.
pub fn draw(o: crate::owners::Owners<'static>) {
    let Some(osk) = crate::input::typed_text::osk() else {
        return;
    };
    let locked_here = o.gfx.locked.surface.get() != 1;
    if locked_here && crate::platform::lock_surface(o.gfx, o.system) < 1 {
        return;
    }
    let mut canvas = GameCanvas::new(o.gfx);
    // The pictures of the controller's buttons for typing, deleting, cancelling and being done.
    let hints = {
        use rc2k_core::input::profile::MenuControl as C;
        use rc2k_core::strings::own::id::HINTS;
        let wanted = [
            (C::Enter, HINTS + 2),
            (C::Third, HINTS + 3),
            (C::Back, HINTS + 4),
        ];
        crate::gfx::button_hints::menu_hints(o, &wanted, Some(HINTS + 5)).map(|(family, hints)| {
            let width = crate::gfx::button_hints::line_width(&hints, &mut canvas);
            (family, hints, width)
        })
    };
    let (w, h) = panel_size(hints.as_ref().map_or(0, |&(_, _, width)| width));
    let left = (SCREEN_W - w) / 2;
    let top = panel_top(field_y(o));
    fill(o, [left, top, w, h], PANEL);
    // What has been typed, in a box over the keys (stars for a password), its end kept in view.
    let (typed, secret) = crate::input::typed_text::osk_echo();
    let box_rect = [
        left + (w - keys_width()) / 2,
        top + MARGIN,
        keys_width(),
        ECHO_H,
    ];
    fill(o, box_rect, ECHO_BOX);
    let mut shown: Vec<u8> = if secret {
        vec![b'*'; typed.len()]
    } else {
        typed
    };
    shown.push(b'_');
    while shown.len() > 1 && width_of(fonts::LIST, &shown, &mut canvas) > box_rect[2] - 16 {
        shown.remove(0);
    }
    shown.push(0);
    let echo_y = box_rect[1] + ECHO_H / 2 - 12;
    if let Err(why) = draw_text_at(
        fonts::LIST,
        &shown[..],
        box_rect[0] + 8,
        echo_y,
        &mut canvas,
    ) {
        crate::program_log::log(&format!("osk: echo not drawn: {why:?}"));
    }
    // The keys sit in the middle of the panel, which is wider than they are when the hints are.
    let keys_left = left + (w - keys_width()) / 2;
    let font = fonts::LIST;
    'rows: for row in 0..osk::ROWS {
        let keys = Osk::width(row) as i32;
        // The command row's keys are wider: the row spans the same width.
        let span = (keys_width() + GAP) / keys;
        for col in 0..keys {
            let key = osk.key_at(row, col as usize);
            let x = keys_left + col * span;
            let y = top + MARGIN + ECHO_H + ECHO_GAP + row as i32 * CELL_H;
            let lit = (row, col as usize) == (osk.row, osk.col);
            let colour = match (lit, key) {
                (true, _) => TILE_LIT,
                (false, Key::Char(_)) => TILE,
                (false, _) => TILE_COMMAND,
            };
            fill(o, [x, y, span - GAP, CELL_H - GAP], colour);
            let text = text_of(&osk, key);
            let tx = x + (span - GAP - width_of(font, &text[..text.len() - 1], &mut canvas)) / 2;
            // The font's own top offset puts its letters a little below the y it is given.
            let ty = y + (CELL_H - GAP) / 2 - 12;
            if let Err(why) = draw_text_at(font, &text[..], tx, ty, &mut canvas) {
                crate::program_log::log(&format!("osk: not drawn: {why:?}"));
                break 'rows;
            }
        }
    }
    if let Some((family, hints, width)) = hints {
        let foot = top + h - MARGIN - crate::gfx::button_hints::SIZE + 2;
        crate::gfx::button_hints::draw_line(o, family, &hints, left + (w - width) / 2, foot);
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
        let (_, h) = panel_size(0);
        // the hint line widens the panel only when it is wider than the keys
        assert_eq!(panel_size(10).0, keys_width() + 2 * MARGIN);
        assert_eq!(panel_size(900).0, 900 + 2 * MARGIN);
        // a field near the top: below it
        assert_eq!(panel_top(Some(220)), 220 + AWAY);
        // a field low down: above it, clear of it
        let top = panel_top(Some(450));
        assert!(top + h <= 450);
        assert_eq!(panel_top(None), DEFAULT_TOP);
    }
}
