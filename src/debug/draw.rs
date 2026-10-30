//! The debug font drawn into the locked frame (`sub_420313` and its parts, IDA 2026-10-30;
//! converted: `rc2k_core::debug::font`): a text at (x, y) in the colour `g_textMode` names
//! (`COLOURS`, made a pixel of `g_surfaceFormat` and kept in `g_debugPixel16` / `32`), each
//! glyph pixel a square of `g_debugScale`; the place the print ended kept in
//! `g_debugCursorX` / `Y`. Refuses a frame not locked (the original writes at address 0).

use crate::errors::{BadInput, Refused};
use crate::gfx::components_to_pixel::components_to_pixel;
use crate::pointers::GamePtr;
use crate::race::numbers_store::NUMBERS;
use crate::race::scratch_store::FRAME_SCRATCH;
use crate::race::text_state_store::TEXT_STATE;
use crate::session::SCREEN;
use rc2k_core::debug::font::{Frame, print_runs};

/// `dword_5C5FEC`: the debug colours (components, one per `g_textMode`); the table's other
/// 238 are 0.
pub const COLOURS: [u32; 18] = [
    0, 0, 0xFFFFFF, 0xFF0000, 0xFF, 0xFF00, 0xFFFF00, 0xFF8000, 0x606060, 0xFFFF, 0xA0A0A0,
    0xFF00FF, 0x4000, 0x909090, 0x400040, 0x80F080, 0x8080F0, 0x400000,
];

/// The colour of text mode `mode` (its low byte).
pub fn colour(mode: u32) -> u32 {
    COLOURS.get((mode & 0xFF) as usize).copied().unwrap_or(0)
}

/// The locked frame's pixels (refused when none is locked: the original writes at 0) and
/// the colour of `g_textMode` made a pixel of the screen's depth, as its bytes.
fn frame_and_pixel() -> Result<(u32, Vec<u8>, u32), Refused> {
    let pixels = FRAME_SCRATCH.frame_pixels.get();
    if pixels == 0 {
        return Err(BadInput("the debug overlay with no frame locked").into());
    }
    let format = crate::gfx::pixel_format::surface_format()?;
    let pixel = components_to_pixel(&format, colour(TEXT_STATE.text_mode.get()));
    let bytes = if SCREEN.bits.get() == 16 {
        (pixel as u16).to_le_bytes().to_vec()
    } else {
        pixel.to_le_bytes().to_vec()
    };
    Ok((pixels, bytes, pixel))
}

/// `sub_420313`: `text` (to its NUL) at (`x`, `y`), or on from the last print at (0, 0).
pub fn print(x: u32, y: u32, text: &[u8]) -> Result<(), Refused> {
    let (pixels, bytes, pixel) = frame_and_pixel()?;
    if bytes.len() == 2 {
        TEXT_STATE.debug_pixel_16.set(pixel);
    } else {
        TEXT_STATE.debug_pixel_32.set(pixel);
    }
    let pitch = FRAME_SCRATCH.frame_pitch.get();
    let frame = Frame {
        pitch,
        wide: bytes.len() as u32,
        len: pitch.wrapping_mul(SCREEN.height.get()),
    };
    let cursor = (
        FRAME_SCRATCH.debug_cursor_x.get(),
        FRAME_SCRATCH.debug_cursor_y.get(),
    );
    let mut runs = Vec::new();
    let (end_x, end_y) = print_runs(
        text,
        (x, y),
        cursor,
        NUMBERS.debug_scale.get(),
        SCREEN.width.get(),
        frame,
        &mut runs,
    );
    for (at, count) in runs {
        GamePtr::new(pixels.wrapping_add(at))
            .write_bytes(&bytes.repeat(count as usize), "the debug font's pixels")?;
    }
    FRAME_SCRATCH.debug_cursor_x.set(end_x);
    FRAME_SCRATCH.debug_cursor_y.set(end_y);
    Ok(())
}

/// `sub_41FF44` (`sub_41FFE5` / `sub_420042`): the rectangle (left, top, right, bottom,
/// inclusive; `dword_5C5FD4` .. `dword_5C5FE0`, here passed in) clamped to the screen and
/// filled in the colour of `g_textMode` (which the original does not mask: past 0xFF refused).
pub fn fill_rect(rect: [u32; 4]) -> Result<(), Refused> {
    use rc2k_core::debug::numbers::clamped;
    if TEXT_STATE.text_mode.get() > 0xFF {
        return Err(BadInput("a debug rectangle's colour past the table").into());
    }
    let Some((left, top, columns, rows)) = clamped(rect, SCREEN.width.get(), SCREEN.height.get())
    else {
        return Ok(());
    };
    let (pixels, bytes, _) = frame_and_pixel()?;
    let pitch = FRAME_SCRATCH.frame_pitch.get();
    let row = bytes.repeat(columns as usize);
    for n in 0..rows {
        let at = (top + n)
            .wrapping_mul(pitch)
            .wrapping_add(left * bytes.len() as u32);
        GamePtr::new(pixels.wrapping_add(at)).write_bytes(&row, "a debug rectangle")?;
    }
    Ok(())
}

/// `g_debugNumberText` (0x5C67A4): the figures' text (0 in the image).
static NUMBER_TEXT: std::sync::Mutex<[u8; rc2k_core::debug::numbers::TEXT_LEN]> =
    std::sync::Mutex::new([0; rc2k_core::debug::numbers::TEXT_LEN]);

/// A figure written into the number text (`write`) and printed at (`x`, `y`).
fn print_figure(
    x: u32,
    y: u32,
    write: impl FnOnce(&mut [u8; rc2k_core::debug::numbers::TEXT_LEN]),
) -> Result<(), Refused> {
    let text = {
        let mut text = NUMBER_TEXT.lock().unwrap_or_else(|e| e.into_inner());
        write(&mut text);
        *text
    };
    print(x, y, &text)
}

/// `sub_4203A3`: `value` in decimal at (`x`, `y`).
pub fn print_whole(x: u32, y: u32, value: u32) -> Result<(), Refused> {
    print_figure(x, y, |t| rc2k_core::debug::numbers::whole(value as i32, t))
}

/// `sub_4203D6`: `value`'s eight hex digits at (`x`, `y`).
pub fn print_hex(x: u32, y: u32, value: u32) -> Result<(), Refused> {
    print_figure(x, y, |t| rc2k_core::debug::numbers::hex(value, t))
}

/// `sub_4203B4`: the float of `bits` as `+ddd.ddd` (or `+dddddd`) at (`x`, `y`): rounded by
/// `fistp`, and times 1000 (`fild`, `fmulp`) as the x87 rounds under this thread's control
/// word. The original's scratch (`g_scratch`) is not written.
pub fn print_fixed(x: u32, y: u32, bits: u32) -> Result<(), Refused> {
    use crate::math::x87_float::{int, ld, to_int};
    let rounded = to_int(ld(bits));
    let thousandths = to_int(ld(bits) * int(1000));
    print_figure(x, y, |t| {
        rc2k_core::debug::numbers::fixed(rounded, thousandths, t)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_number_text_is_the_images() {
        let image =
            crate::race::store::first_values::<{ rc2k_core::debug::numbers::TEXT_LEN }>(0x5C67A4);
        assert!(image.iter().all(|&b| b == 0));
    }

    #[test]
    fn the_colours_are_the_images() {
        let image = crate::race::store::first_values::<{ 256 * 4 }>(0x5C5FEC);
        for (n, word) in image.chunks(4).enumerate() {
            let word = u32::from_le_bytes(word.try_into().unwrap());
            assert_eq!(word, colour(n as u32), "colour {n}");
        }
    }

    #[test]
    fn the_glyphs_are_the_images() {
        use rc2k_core::debug::font::{GLYPH_COUNT, GLYPHS, ROWS};
        let image = crate::race::store::first_values::<{ ROWS * GLYPH_COUNT * 2 }>(0x5C6614);
        for (n, word) in image.chunks(2).enumerate() {
            let word = u16::from_le_bytes(word.try_into().unwrap());
            assert_eq!(word, GLYPHS[n / GLYPH_COUNT][n % GLYPH_COUNT], "word {n}");
        }
    }
}
