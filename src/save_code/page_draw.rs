//! What the save code page and its question are drawn with: the game's menu font on a plain
//! screen, panels and buttons as the error dialog has them, and the QR code as filled
//! rectangles of whole pixels (black on white, with its quiet zone).

use crate::gfx::button_hints::width_of;
use crate::gfx::typefaces::fonts;
use crate::input::osk_draw::fill;
use crate::owners::Owners;
use crate::text::draw_text::{GameCanvas, draw_text_at};
use rc2k_core::saves::code::qr::{self, Modules};

/// Colours (0xRRGGBB): the plain screen, a panel, a button, the one chosen, one that cannot be
/// used, and the code's two.
pub const SCREEN: u32 = 0x05081A;
pub const PANEL: u32 = 0x0C1238;
pub const BUTTON: u32 = 0x27347A;
pub const CHOSEN: u32 = 0xC8202A;
pub const UNUSABLE: u32 = 0x14183A;
const WHITE: u32 = 0xFF_FFFF;
const BLACK: u32 = 0;

pub const MARGIN: i32 = 16;
pub const LINE_H: i32 = 30;
pub const BUTTON_H: i32 = 38;
/// The font's own top offset puts its letters a little below the y they are given.
const TEXT_UP: i32 = 12;

/// The screen's size.
pub fn screen_size(o: Owners<'static>) -> (i32, i32) {
    let screen = o.system.session.screen.get();
    (screen.width.get() as i32, screen.height.get() as i32)
}

/// A text's width in the menu font.
pub fn width(o: Owners<'static>, text: &str) -> i32 {
    width_of(
        fonts::LIST,
        &rc2k_core::files::ansi::ansi_bytes(text),
        &mut GameCanvas::new(o.gfx),
    )
}

/// `text` with its middle line at `y`.
pub fn text(o: Owners<'static>, text: &str, x: i32, y: i32) {
    let mut bytes = rc2k_core::files::ansi::ansi_bytes(text);
    bytes.push(0);
    // The fonts not being there yet is the only failure, and nothing is shown then.
    let _ = draw_text_at(
        fonts::LIST,
        &bytes[..],
        x,
        y - TEXT_UP,
        &mut GameCanvas::new(o.gfx),
    );
}

/// `text` centred in the span from `x` of `w` pixels.
pub fn centred(o: Owners<'static>, s: &str, x: i32, w: i32, y: i32) {
    text(o, s, x + (w - width(o, s)) / 2, y);
}

/// `text` cut with "..." where it passes `room` pixels.
pub fn fit(o: Owners<'static>, text: &str, room: i32) -> String {
    if width(o, text) <= room {
        return text.to_owned();
    }
    let mut cut: Vec<char> = text.chars().collect();
    while !cut.is_empty() {
        cut.pop();
        let tried: String = cut.iter().collect::<String>() + "...";
        if width(o, &tried) <= room {
            return tried;
        }
    }
    String::new()
}

/// A button: the label in the middle of `rect`.
pub fn button(o: Owners<'static>, rect: [i32; 4], label: &str, chosen: bool, usable: bool) {
    let colour = match (chosen, usable) {
        (true, true) => CHOSEN,
        (_, true) => BUTTON,
        (true, false) => 0x3A2230,
        (false, false) => UNUSABLE,
    };
    fill(o, rect, colour);
    let [x, y, w, h] = rect;
    centred(o, label, x, w, y + h / 2);
}

/// The text cut into lines of `per_line` letters.
pub fn lines_of(text: &str, per_line: usize) -> impl Iterator<Item = &str> {
    text.as_bytes()
        .chunks(per_line.max(1))
        // The text is of ASCII letters (the code's alphabet), so a cut is always between letters.
        .map(|c| std::str::from_utf8(c).unwrap_or(""))
}

/// How many letters of the code fit a line of `room` pixels: its widest letter decides.
pub fn letters_per_line(o: Owners<'static>, room: i32) -> usize {
    let widest = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:"
        .chars()
        .map(|c| width(o, &c.to_string()))
        .max()
        .unwrap_or(1)
        .max(1);
    (room / widest).max(1) as usize
}

/// The code at (`x`, `y`), a module `scale` pixels square, black on white with its quiet zone.
pub fn qr_code(o: Owners<'static>, modules: &Modules, x: i32, y: i32, scale: usize) {
    let s = scale as i32;
    let zone = qr::QUIET as i32 * s;
    let full = modules.side as i32 * s + 2 * zone;
    fill(o, [x, y, full, full], WHITE);
    for row in 0..modules.side {
        let mut col = 0;
        while col < modules.side {
            if !modules.at(col, row) {
                col += 1;
                continue;
            }
            let start = col;
            while col < modules.side && modules.at(col, row) {
                col += 1;
            }
            fill(
                o,
                [
                    x + zone + start as i32 * s,
                    y + zone + row as i32 * s,
                    (col - start) as i32 * s,
                    s,
                ],
                BLACK,
            );
        }
    }
}

/// The camera's picture as blocks of colour, `cells` across, in the box at (`x`, `y`) of
/// `room` pixels' width.
pub fn preview(
    o: Owners<'static>,
    frame: &crate::platform::ffi::sdl_camera::Frame,
    x: i32,
    y: i32,
    room: i32,
) {
    let cells = (room / 2).clamp(40, 120) as usize;
    let rows = (cells * frame.height / frame.width).max(1);
    // The cells' edges at their share of the box, so the picture fills it whatever the size.
    let edge = |n: usize, of: usize| {
        (n as i32 * room * frame.height as i32 / frame.width as i32 / of as i32).max(0)
    };
    let across = |n: usize| n as i32 * room / cells as i32;
    for cy in 0..rows {
        for cx in 0..cells {
            let (sx, sy) = (cx * frame.width / cells, cy * frame.height / rows);
            let at = (sy * frame.width + sx) * 4;
            let rgb = u32::from(frame.rgba[at]) << 16
                | u32::from(frame.rgba[at + 1]) << 8
                | u32::from(frame.rgba[at + 2]);
            let (x0, y0) = (across(cx), edge(cy, rows));
            fill(
                o,
                [x + x0, y + y0, across(cx + 1) - x0, edge(cy + 1, rows) - y0],
                rgb,
            );
        }
    }
}
