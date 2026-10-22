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
use crate::framework::{game_addr, game_ptr, game_read, rva};
use crate::gfx::blend_palette::blend_palette;
use crate::gfx::pixel_format::{SCRATCH_RVA, surface_format};
use crate::service::harness::{Region, at};
use crate::text::font::Font;

// ---- game data ----

pub const BLENDED_FONT_RVA: usize = rva(0x51E384); // stru_51E384
const BLENDED_PALETTE_RVA: usize = rva(0x4F3B74); // dword_4F3B74
const BLEND_MODE_RVA: usize = rva(0x4F3F74); // dword_4F3F74: 0 = no palette is made, 2 = eased
const PALETTE_ENTRIES: u32 = 15;
const FONT_LEN: usize = 0x20;

/// Everything `blend_font` may write.
pub fn blend_regions() -> [Region; 3] {
    [
        (game_addr(BLENDED_FONT_RVA) as u32, FONT_LEN),
        (
            game_addr(BLENDED_PALETTE_RVA) as u32,
            PALETTE_ENTRIES as usize * 4,
        ),
        at(SCRATCH_RVA),
    ]
}

// ---- Rust version ----

pub unsafe fn blend_font(
    time: u32,
    half_pulse: u32,
    a: *const Font,
    b: *const Font,
) -> Result<(), BadInput> {
    if a.is_null() || b.is_null() {
        return Err(BadInput("a null font"));
    }
    let mode: u32 = unsafe { game_read(BLEND_MODE_RVA) };
    // Converted: `rc2k_core::text::pulse`.
    let (from_a, t) = rc2k_core::text::pulse(time, half_pulse, mode == 2);
    let (from, to) = if from_a { (a, b) } else { (b, a) };
    unsafe {
        let blended: *mut Font = game_ptr(BLENDED_FONT_RVA);
        // The original reads the palettes after the copy, so if the second font is the
        // blended font itself, its palette is by then the first one's.
        let from_palette = from.read_unaligned().palette;
        let to_palette = if std::ptr::eq(to, blended) {
            from_palette
        } else {
            to.read_unaligned().palette
        };
        if mode != 0 && (from_palette == 0 || to_palette == 0) {
            return Err(BadInput("a font without a palette"));
        }
        std::ptr::copy(from as *const u8, blended as *mut u8, FONT_LEN);
        if mode == 0 {
            return Ok(());
        }
        let palette: *mut u32 = game_ptr(BLENDED_PALETTE_RVA);
        blend_palette(
            from_palette as *const u32,
            to_palette as *const u32,
            palette,
            PALETTE_ENTRIES,
            t,
            &surface_format(),
        )?;
        let mut font = blended.read_unaligned();
        font.palette = palette as u32;
        blended.write_unaligned(font);
    }
    Ok(())
}
