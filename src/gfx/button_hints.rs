//! A line of hints: each a button picture and a word beside it ("A  type  X  delete ..."), for
//! the on-screen keyboard and the race's leave question. The controller the pictures are of is
//! the current player's (`pictures_in_use`).

use crate::gfx::typefaces::{FontId, fonts};
use crate::text::draw_text::{Canvas, GameCanvas, draw_text_at};
use rc2k_core::input::icons::{Family, Icon};
use rc2k_core::input::profile::{Kind, MenuControl};

/// A picture's side in a hint line.
pub const SIZE: i32 = 32;
/// The gap between a picture and its word, and between hints.
const BESIDE: i32 = 2;
const BETWEEN: i32 = 20;
/// The font's own top offset puts its letters a little below the y it is given.
const TEXT_UP: i32 = 12;

/// A hint: a picture and its word (the word's bytes, no NUL).
pub struct Hint {
    pub icon: Icon,
    pub word: Vec<u8>,
}

/// A text's width in a font, measured without drawing.
pub fn width_of(font: FontId, text: &[u8], canvas: &mut GameCanvas) -> i32 {
    let mut copy = font.object(canvas.typefaces()).get();
    copy.mode = rc2k_core::strings::text::MODE_MEASURE;
    let mut nul = text.to_vec();
    nul.push(0);
    crate::text::measure_text::measure_text(&mut copy, &nul[..], 0, canvas)
        .map(|m| m.width as i32)
        .unwrap_or(0)
}

/// The pictures of the controller in use: the current player's, or the first controller when
/// the player has none; with the profile's bindings. None when the player has the buttons as
/// numbers, or the controller is not known.
pub fn pictures_in_use(
    o: crate::owners::Owners<'static>,
) -> Option<(
    rc2k_core::input::icons::Pictures,
    rc2k_core::input::profile::Bindings,
)> {
    let current = o.menus.screens.event.read().current_player;
    let controller = crate::menu::player::player(o.menus, current)
        .map_or(0, |p| p.get().controller)
        .max(1);
    crate::input::profiles::pictures(o.input, current, controller)
}

/// The hints for what the controller in use does in the menus: the menu control, its word's own
/// text id, in the order given; the controls the controller has no picture for are left out.
/// `start` adds the pad's Start (with its word) at the end for a gamepad.
pub fn menu_hints(
    o: crate::owners::Owners<'static>,
    controls: &[(MenuControl, u32)],
    start: Option<u32>,
) -> Option<(Family, Vec<Hint>)> {
    let (pictures, bindings) = pictures_in_use(o)?;
    let word = |id| crate::strings::append_string_by_id::own_text(o.strings, id);
    let mut hints: Vec<Hint> = controls
        .iter()
        .filter_map(|&(control, id)| {
            let icon = pictures.menu(control, bindings.menu(control))?;
            crate::gfx::button_pictures::has(pictures.family, icon).then(|| Hint {
                icon,
                word: word(id),
            })
        })
        .collect();
    if let (Some(id), Kind::Pad) = (start, pictures.kind) {
        hints.push(Hint {
            icon: Icon::Start,
            word: word(id),
        });
    }
    (!hints.is_empty()).then_some((pictures.family, hints))
}

/// The line's width.
pub fn line_width(hints: &[Hint], canvas: &mut GameCanvas) -> i32 {
    hints
        .iter()
        .map(|h| SIZE + BESIDE + width_of(fonts::LIST, &h.word, canvas))
        .sum::<i32>()
        + BETWEEN * (hints.len() as i32 - 1).max(0)
}

/// The hints drawn in a line from `left`, its pictures' top at `top`. The surface must be
/// locked.
pub fn draw_line(
    o: crate::owners::Owners<'static>,
    family: Family,
    hints: &[Hint],
    left: i32,
    top: i32,
) {
    let mut canvas = GameCanvas::new(o.gfx);
    let mut x = left;
    for hint in hints {
        draw_beside_text(o, family, hint.icon, x, top + SIZE / 2 - TEXT_UP, SIZE);
        x += SIZE + BESIDE;
        let mut text = hint.word.clone();
        text.push(0);
        let _ = draw_text_at(
            fonts::LIST,
            &text[..],
            x,
            top + SIZE / 2 - TEXT_UP,
            &mut canvas,
        );
        x += width_of(fonts::LIST, &hint.word, &mut canvas) + BETWEEN;
    }
}

/// The rows below the line's top that `text`'s first letter covers in `font`: (the first, how many).
pub fn capital_rows(
    font: &rc2k_core::strings::text::Font,
    text: &[u8],
    canvas: &crate::text::draw_text::GameCanvas,
) -> Option<(i32, i32)> {
    let c = *text.iter().find(|&&c| c > b' ')?;
    if c < font.first_char || c > font.last_char {
        return None;
    }
    let glyphs =
        crate::gfx::glyph_store::bytes(canvas.fonts(), font.glyph_table, "the question's font")
            .ok()?;
    let (blank, rows) =
        rc2k_core::strings::text::ink_rows(&glyphs, u32::from(c - font.first_char)).ok()?;
    (rows > 0).then_some((font.y_offset + blank, rows))
}

/// Picture `icon` drawn in a cell `size` high with its outline centred on the capitals of a
/// text drawn in the menus' list font with its y at `text_y`, its left at `x`. The surface must
/// be locked.
pub fn draw_beside_text(
    o: crate::owners::Owners<'static>,
    family: Family,
    icon: Icon,
    x: i32,
    text_y: i32,
    size: i32,
) -> bool {
    let canvas = GameCanvas::new(o.gfx);
    let font = fonts::LIST.object(canvas.typefaces()).get();
    let middle = capital_rows(&font, b"H", &canvas)
        .map_or(text_y + size / 2, |(top, rows)| text_y + top + rows / 2);
    let (from, to) = crate::gfx::button_pictures::outline_rows(family, icon).unwrap_or((0.0, 1.0));
    let top = middle - (size as f32 * (from + to) / 2.0).round() as i32;
    crate::gfx::button_pictures::draw_glyph(o, family, icon, x, top, size)
}
