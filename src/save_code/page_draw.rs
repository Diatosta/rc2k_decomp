//! What the save code page and its question are drawn with: the game's menu font on a plain
//! screen, panels and buttons as the error dialog has them, and the QR code as filled
//! rectangles of whole pixels (black on white, with its quiet zone).

use crate::gfx::button_hints::width_of;
use crate::gfx::typefaces::fonts;
use crate::input::osk_draw::fill;
use crate::owners::Owners;
use crate::platform::pointer;
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
/// A button's height: on a touch screen 1.9 lines, tall enough for a finger.
pub const BUTTON_H: i32 = pointer::by_style(38, 56);
/// The space between buttons side by side (a touch screen's are further apart).
pub const BUTTON_GAP: i32 = pointer::by_style(MARGIN / 2, 12);
/// The space between two buttons one above the other: half a button on a touch screen.
pub const WAYS_GAP: i32 = pointer::by_style(6, BUTTON_H / 2);
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
/// Written a pixel row at a time into the surface (a code of the largest size has tens of
/// thousands of runs, which drawn as filled rectangles cost every frame what the page cannot
/// spare); a box that does not lie inside the surface is not drawn.
pub fn qr_code(o: Owners<'static>, modules: &Modules, x: i32, y: i32, scale: usize) {
    let surface = crate::gfx::surface::surface(o.gfx);
    let Some(target) = crate::gfx::surface::target(o.gfx) else {
        return;
    };
    let (screen_w, screen_h) = screen_size(o);
    let bytes = usize::try_from(surface.bytes_per_pixel).unwrap_or(0);
    let zone = qr::QUIET * scale;
    let full = (modules.side + 2 * qr::QUIET) * scale;
    if bytes == 0
        || x < 0
        || y < 0
        || x as usize + full > screen_w as usize
        || y as usize + full > screen_h as usize
    {
        return;
    }
    let paint = |rgb: u32| {
        crate::input::osk_draw::pixel(rgb, surface.bits_per_pixel).to_le_bytes()[..bytes].to_vec()
    };
    let (white, black) = (paint(WHITE), paint(BLACK));
    let blank_row = white.repeat(full);
    let mut row = Vec::with_capacity(full * bytes);
    for line in 0..full {
        // The module row this pixel row belongs to, none in the quiet zone.
        let module_row = (line / scale)
            .checked_sub(qr::QUIET)
            .filter(|r| *r < modules.side);
        let pixels = match module_row {
            None => &blank_row,
            Some(r) => {
                row.clear();
                row.extend_from_slice(&white.repeat(zone));
                for col in 0..modules.side {
                    let colour = if modules.at(col, r) { &black } else { &white };
                    for _ in 0..scale {
                        row.extend_from_slice(colour);
                    }
                }
                row.extend_from_slice(&white.repeat(zone));
                &row
            }
        };
        let offset = (y as u32 + line as u32) * surface.pitch as u32 + x as u32 * bytes as u32;
        // The box was kept inside the surface above; a refusal would only leave a row blank.
        let _ = crate::gfx::surface::write(target, offset, pixels, o.gfx);
    }
}

/// The camera's picture, all of it, in a box `room` pixels wide at `at` (the frame's own
/// proportions), mirrored when the camera faces the user (as a mirror shows). Its pixels are
/// written into the surface a row at a time.
pub fn preview(
    o: Owners<'static>,
    frame: &crate::platform::ffi::sdl_camera::Frame,
    at: [i32; 2],
    room: i32,
) {
    let surface = crate::gfx::surface::surface(o.gfx);
    let Some(target) = crate::gfx::surface::target(o.gfx) else {
        return;
    };
    let (screen_w, screen_h) = screen_size(o);
    let to_width = room.clamp(1, screen_w) as usize;
    let to_height = (room as i64 * frame.height as i64 / frame.width.max(1) as i64).max(1) as usize;
    let pixels = rc2k_core::gfx::camera_frame::fit_rgb(
        &frame.rgba,
        frame.width,
        frame.height,
        to_width,
        to_height,
        frame.facing_user,
    );
    let bytes = usize::try_from(surface.bytes_per_pixel).unwrap_or(0);
    if pixels.is_empty() || bytes == 0 || at[0] < 0 || at[0] as usize + to_width > screen_w as usize
    {
        return;
    }
    let mut row = Vec::with_capacity(to_width * bytes);
    for (n, line) in pixels.chunks_exact(to_width).enumerate() {
        let y = at[1] + n as i32;
        if y < 0 || y >= screen_h {
            continue;
        }
        row.clear();
        for &rgb in line {
            let value = crate::input::osk_draw::pixel(rgb, surface.bits_per_pixel);
            row.extend_from_slice(&value.to_le_bytes()[..bytes]);
        }
        let offset = y as u32 * surface.pitch as u32 + at[0] as u32 * bytes as u32;
        // The box was kept inside the surface above; a refusal would only leave a row blank.
        let _ = crate::gfx::surface::write(target, offset, &row, o.gfx);
    }
}
