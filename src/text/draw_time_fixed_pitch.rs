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

use crate::errors::Refused;
use crate::text::draw_text::Canvas;
use crate::text::draw_text_keep_regs::draw_text_keep_regs;
use crate::text::font::Font;
use rc2k_core::text::{TIME_STEPS, columns};

// ---- Rust version ----

/// Where the one-character text is built, relative to the text.
pub const CELL_OFFSET: usize = 0x40;

/// Returns whether the last character was too wide: the carry flag.
pub fn draw_time_fixed_pitch(
    font: &mut Font,
    text: crate::pointers::GamePtr,
    x: i32,
    y: i32,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    let mut too_wide = false;
    let cell = text.offset(CELL_OFFSET as u32);
    // Converted: `rc2k_core::text::{columns, TIME_STEPS}`.
    for (i, x) in columns(x, &TIME_STEPS).into_iter().enumerate() {
        put_cell(text, i)?;
        too_wide = draw_text_keep_regs(font, cell, x, y, canvas)?;
    }
    Ok(too_wide)
}

/// Copies the text's byte `i` to its cell with a NUL after it.
pub fn put_cell(text: crate::pointers::GamePtr, i: usize) -> Result<(), Refused> {
    let b = text.offset(i as u32).read::<u8>("a fixed-pitch text")?;
    text.offset(CELL_OFFSET as u32)
        .write_bytes(&[b, 0], "a fixed-pitch text's cell")?;
    Ok(())
}

/// The same with a font object in game memory, for Rust callers.
pub fn draw_time_fixed_pitch_at(
    font: crate::pointers::GamePtr,
    text: crate::pointers::GamePtr,
    x: i32,
    y: i32,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    let before = font.read::<Font>("a font")?;
    let mut copy = before;
    let too_wide = draw_time_fixed_pitch(&mut copy, text, x, y, canvas)?;
    if copy != before && canvas.keeps_font_changes() {
        font.write(copy, "a font")?;
    }
    Ok(too_wide)
}
