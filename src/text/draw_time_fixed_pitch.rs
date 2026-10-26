//! `text::draw_time_fixed_pitch` (0x465EC4): draws the first eight bytes of a text one at a
//! time in fixed columns, so that the digits of a time like "01:23.45" do not move.
//!
//! In:  eax = font, ebx = text, ecx = x of the right edge, edx = y
//! Out: carry as the last `draw_text` leaves it; every register preserved
//! Each byte is copied to text + 0x40 with a NUL after it and drawn from there, so the text
//! needs 0x42 bytes of room. All eight bytes are read even if a NUL comes earlier. The columns
//! are at x - 100 plus 0, 13, 23, 33, 46, 56, 66 and 79.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::FatalError;
use crate::text::draw_text::Canvas;
use crate::text::draw_text_keep_regs::draw_text_keep_regs;
use crate::text::font::Font;
use rc2k_core::text::{TIME_STEPS, columns};

// ---- Rust version ----

/// Where the one-character text is built, relative to the text.
pub const CELL_OFFSET: usize = 0x40;

/// Returns whether the last character was too wide: the carry flag.
pub unsafe fn draw_time_fixed_pitch(
    font: &mut Font,
    text: *mut u8,
    x: i32,
    y: i32,
    canvas: &mut dyn Canvas,
) -> Result<bool, FatalError> {
    let mut too_wide = false;
    unsafe {
        let cell = text.add(CELL_OFFSET);
        // Converted: `rc2k_core::text::{columns, TIME_STEPS}`.
        for (i, x) in columns(x, &TIME_STEPS).into_iter().enumerate() {
            *cell = *text.add(i);
            *cell.add(1) = 0;
            too_wide = draw_text_keep_regs(font, cell, x, y, canvas)?;
        }
    }
    Ok(too_wide)
}

/// The same with a font object in game memory, for Rust callers.
pub unsafe fn draw_time_fixed_pitch_at(
    font: *mut Font,
    text: *mut u8,
    x: i32,
    y: i32,
    canvas: &mut dyn Canvas,
) -> Result<bool, FatalError> {
    unsafe {
        let before = font.read_unaligned();
        let mut copy = before;
        let too_wide = draw_time_fixed_pitch(&mut copy, text, x, y, canvas)?;
        if copy != before && canvas.keeps_font_changes() {
            font.write_unaligned(copy);
        }
        Ok(too_wide)
    }
}
