//! `text::blend_font` (0x405E92): makes the font a highlighted menu item is drawn with, whose
//! colours pulse between those of two fonts.
//!
//! In:  ecx = font A, edx = font B, eax = the time, ebx = the length of a half pulse (0x100)
//! Out: every register preserved
//! In the half pulses where `eax & ebx` is not 0 the blend goes from A towards B, in the
//! others from B towards A. The font it starts from is copied (0x20 bytes) to the blended
//! font `stru_51E384`. Unless `dword_4F3F74` is 0, a 15-entry palette is then made in
//! `dword_4F3B74` with `gfx::blend_palette` from the two fonts' palettes, `eax & (ebx - 1)`
//! of the way (with `dword_4F3F74` = 2 that amount t is eased: t * t >> 8, then the same on
//! 255 - t), and the blended font is given that palette.
//! Two callers: the menu items and `sub_4695C6`.
//! Refuses null fonts, and null palettes when a palette is made.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::gfx::blend_palette::blend_palette;
use crate::gfx::pixel_format::surface_format;
use crate::moved::Place;
use crate::pointers::GamePtr;
use crate::text::font::Font;
use std::sync::atomic::{AtomicU32, Ordering};

// ---- game data ----

/// g_blendedFont (stru_51E384, 0x20 bytes, out of the image since 2026-10-24, phase 5): the
/// font a highlighted item is drawn with. Its first bytes are the image's.
pub static BLENDED_FONT: Place<[u32; 8]> = Place::new([0x0400_FF20, 1, 0x1B, 0, 6, 0, 0, 0]);
/// g_blendedPalette (0x4F3B74, 1024 bytes of which 15 entries are made): 0 at the start.
pub static BLENDED_PALETTE: Place<[u32; 256]> = Place::new([0; 256]);
/// g_fontBlendMode (0x4F3F74, the setting `textflashmode`, through `crate::moved::place`):
/// 0 = no palette is made, 2 = eased. 3 in the image.
pub static BLEND_MODE: AtomicU32 = AtomicU32::new(3);
const PALETTE_ENTRIES: u32 = 15;
const FONT_LEN: usize = 0x20;

// ---- Rust version ----

pub fn blend_font(time: u32, half_pulse: u32, a: GamePtr, b: GamePtr) -> Result<(), BadInput> {
    if a.addr() == 0 || b.addr() == 0 {
        return Err(BadInput("a null font"));
    }
    let mode: u32 = BLEND_MODE.load(Ordering::Relaxed);
    // Converted: `rc2k_core::text::pulse`.
    let (from_a, t) = rc2k_core::text::pulse(time, half_pulse, mode == 2);
    let (from, to) = if from_a { (a, b) } else { (b, a) };
    let blended = GamePtr::to(&BLENDED_FONT);
    // The original reads the palettes after the copy, so if the second font is the
    // blended font itself, its palette is by then the first one's.
    let from_palette = from.read::<Font>("a font")?.palette;
    let to_palette = if to == blended {
        from_palette
    } else {
        to.read::<Font>("a font")?.palette
    };
    if mode != 0 && (from_palette == 0 || to_palette == 0) {
        return Err(BadInput("a font without a palette"));
    }
    from.copy_to(blended, FONT_LEN, "a font")?;
    if mode == 0 {
        return Ok(());
    }
    let palette = GamePtr::to(&BLENDED_PALETTE);
    blend_palette(
        from_palette,
        to_palette,
        palette,
        PALETTE_ENTRIES,
        t,
        &surface_format()?,
    )?;
    let mut font = blended.read::<Font>("the blended font")?;
    font.palette = palette.addr();
    blended.write(font, "the blended font")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::moved::image_bytes;

    #[test]
    fn data_starts_as_the_games() {
        let font = unsafe { *super::BLENDED_FONT.ptr() };
        let bytes: Vec<u8> = font.iter().flat_map(|w| w.to_le_bytes()).collect();
        assert_eq!(bytes, image_bytes(0x51E384, 0x20));
        let palette = unsafe { *super::BLENDED_PALETTE.ptr() };
        let bytes: Vec<u8> = palette.iter().flat_map(|w| w.to_le_bytes()).collect();
        assert_eq!(bytes, image_bytes(0x4F3B74, 0x400));
        let mode = super::BLEND_MODE.load(std::sync::atomic::Ordering::Relaxed);
        assert_eq!(mode.to_le_bytes().to_vec(), image_bytes(0x4F3F74, 4));
    }
}
