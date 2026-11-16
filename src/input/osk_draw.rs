//! Draws the on-screen keyboard (`rc2k_core::input::osk`) over the frame about to be shown, with
//! the menus' fonts through the text drawer, on both renderers.

use crate::gfx::typefaces::fonts;
use crate::text::draw_text::{GameCanvas, draw_text_at};
use rc2k_core::input::osk::{self, Key, Osk};

/// The keys' cell and the keyboard's place on the menus' screen.
const CELL_W: i32 = 36;
const CELL_H: i32 = 24;
const LEFT: i32 = 140;
const TOP: i32 = 290;
/// The space round the keys inside the panel, and the panel's entry in the fonts' palette.
const PANEL_MARGIN: i32 = 8;
const PANEL_COLOUR: u32 = 0;

fn text_of(osk: &Osk, row: usize, col: usize) -> Vec<u8> {
    let mut text = match osk.key_at(row, col) {
        Key::Char(c) => vec![c],
        key => osk::label(key, osk.shifted).to_vec(),
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
    // A dark panel behind the keys, so the menu's own items under it do not show through; drawn
    // row by row, as the rectangle fill draws nothing on a 32-bit screen.
    let palette = crate::gfx::rect::Palette::Held(o.gfx.fonts.font_block.palettes.get());
    let width = CELL_W * osk::COLUMNS as i32 + 2 * PANEL_MARGIN;
    let height = CELL_H * osk::ROWS as i32 + 2 * PANEL_MARGIN;
    for y in 0..height {
        crate::gfx::rect::draw_hline(
            LEFT - PANEL_MARGIN,
            TOP - PANEL_MARGIN + y,
            width,
            PANEL_COLOUR,
            palette,
            o.gfx,
        );
    }
    let mut canvas = GameCanvas::new(o.gfx);
    for row in 0..osk::ROWS {
        let keys = Osk::width(row);
        // The command row's keys are wider: the row spans the same width.
        let cell = CELL_W * osk::COLUMNS as i32 / keys as i32;
        for col in 0..keys {
            let font = if (row, col) == (osk.row, osk.col) {
                fonts::BUTTON_LIT
            } else {
                fonts::LIST
            };
            let x = LEFT + col as i32 * cell + 6;
            let y = TOP + row as i32 * CELL_H;
            if let Err(why) = draw_text_at(font, &text_of(&osk, row, col)[..], x, y, &mut canvas) {
                crate::logfile::program_log::log(&format!("osk: not drawn: {why:?}"));
                break;
            }
        }
    }
    if locked_here {
        crate::platform::unlock_surface(o.gfx);
    }
}
