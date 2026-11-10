//! `text::blend_font` (0x405E92): makes the font a highlighted menu item is drawn with, whose
//! colours pulse between those of two fonts.
//!
//! Takes: font A, font B, the time, the length of a half pulse (0x100)
//! In the half pulses where `eax & ebx` is not 0 the blend goes from A towards B, in the
//! others from B towards A. The font it starts from is copied (0x20 bytes) to the blended
//! font `stru_51E384`. Unless `dword_4F3F74` is 0, a 15-entry palette is then made in
//! `dword_4F3B74` with `gfx::blend_palette` from the two fonts' palettes, `eax & (ebx - 1)`
//! of the way (with `dword_4F3F74` = 2 that amount t is eased: t * t >> 8, then the same on
//! 255 - t), and the blended font is given that palette.
//! Two callers: the menu items and `screens::draw_highlighted_caption`.
//! Refuses null fonts, and null palettes when a palette is made.

use crate::errors::BadInput;
use crate::gfx::blend_palette::blend_palette;
use crate::gfx::owner::gfx;
use crate::gfx::pixel_format::surface_format;
use crate::gfx::typefaces::{FontObject, FontRef};
use crate::pointers::GamePtr;
use crate::session::Dword;
use crate::strings::owner::strings;
use crate::text::font::Font;

// ---- game data ----

/// g_blendedFont (stru_51E384, 0x20 bytes, out of the image since 2026-10-24, phase 5): the
/// font a highlighted item is drawn with. Its first bytes are the image's.
pub static BLENDED_FONT: FontObject = FontObject::from_font(Font {
    first_char: 0x20,
    last_char: 0xFF,
    mode: 0,
    flags: 4,
    spacing: 1,
    line_height: 0x1B,
    y_offset: 0,
    extended_spacing: 6,
    palette: 0,
    glyph_table: 0,
    box_width: 0,
});
/// g_blendedPalette (0x4F3B74, 1024 bytes of which 15 entries are made): 0 at the start.
pub static BLENDED_PALETTE: [Dword; 256] = [const { Dword::new(0) }; 256];
const PALETTE_ENTRIES: u32 = 15;

pub fn blend_font(
    time: u32,
    half_pulse: u32,
    a: impl Into<FontRef>,
    b: impl Into<FontRef>,
) -> Result<(), BadInput> {
    let (a, b) = (a.into(), b.into());
    let mode: u32 = strings().text.blend_mode.get();
    // Converted: `rc2k_core::text::pulse`.
    let (from_a, t) = rc2k_core::text::pulse(time, half_pulse, mode == 2);
    let (from, to) = if from_a { (a, b) } else { (b, a) };
    // The original reads the palettes after the copy, so if the second font is the
    // blended font itself, its palette is by then the first one's.
    let from_font = from.object().get();
    let to_palette = if to.is_blended() {
        from_font.palette
    } else {
        to.object().get().palette
    };
    if mode != 0 && (from_font.palette == 0 || to_palette == 0) {
        return Err(BadInput("a font without a palette"));
    }
    BLENDED_FONT.set(from_font);
    if mode == 0 {
        return Ok(());
    }
    let palette = GamePtr::to(&BLENDED_PALETTE);
    blend_palette(
        &gfx().pixel.formats,
        from_font.palette,
        to_palette,
        palette,
        PALETTE_ENTRIES,
        t,
        &surface_format()?,
    )?;
    BLENDED_FONT.palette.set(palette.addr());
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::moved::image_bytes;
    use crate::strings::owner::strings;

    #[test]
    fn data_starts_as_the_games() {
        let f = super::BLENDED_FONT.get();
        let mut bytes = vec![f.first_char, f.last_char, f.mode, f.flags];
        for w in [
            f.spacing as u32,
            f.line_height as u32,
            f.y_offset as u32,
            f.extended_spacing as u32,
            f.palette,
            f.glyph_table,
            f.box_width,
        ] {
            bytes.extend(w.to_le_bytes());
        }
        assert_eq!(bytes, image_bytes(0x51E384, 0x20));
        let bytes: Vec<u8> = super::BLENDED_PALETTE
            .iter()
            .flat_map(|w| w.get().to_le_bytes())
            .collect();
        assert_eq!(bytes, image_bytes(0x4F3B74, 0x400));
        let mode = strings().text.blend_mode.get();
        assert_eq!(mode.to_le_bytes().to_vec(), image_bytes(0x4F3F74, 4));
    }
}
