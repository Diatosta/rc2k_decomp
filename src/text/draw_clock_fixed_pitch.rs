//! `text::draw_clock_fixed_pitch` (0x465F42): as `text::draw_time_fixed_pitch` for a time
//! of day: the first five bytes of a text ("09:05") one at a time in fixed columns, at x -
//! 100 plus 0, 13, 23, 33 and 46.
//!
//! In:  eax = font, ebx = text, ecx = x of the right edge, edx = y
//! Out: the flags of the original's last `add ecx, 0Ah` (after the fifth character):
//!      carry when x - 54 + 10 passes 2^32, zero when it is 0; every register preserved
//! Each byte is copied to text + 0x40 with a NUL after it and drawn from there. Three
//! callers (the itinerary pages).
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::Refused;
use crate::text::draw_text::Canvas;
use crate::text::draw_text_keep_regs::draw_text_keep_regs;
use crate::text::font::Font;
use rc2k_core::text::{CLOCK_STEPS, columns};

// ---- Rust version ----

/// Where the one-character text is built, relative to the text.
pub const CELL_OFFSET: usize = crate::text::draw_time_fixed_pitch::CELL_OFFSET;

/// Returns the flags the original leaves: (carry, zero) of its last `add` (draw_text's
/// own carry is lost there).
pub unsafe fn draw_clock_fixed_pitch(
    font: &mut Font,
    text: *mut u8,
    x: i32,
    y: i32,
    canvas: &mut dyn Canvas,
) -> Result<(bool, bool), Refused> {
    let at = columns(x, &CLOCK_STEPS);
    unsafe {
        let cell = text.add(CELL_OFFSET);
        // Converted: `rc2k_core::text::{columns, CLOCK_STEPS}`.
        for (i, &x) in at.iter().enumerate() {
            *cell = *text.add(i);
            *cell.add(1) = 0;
            draw_text_keep_regs(font, cell, x, y, canvas)?;
        }
    }
    // The flags of the original's last `add`, after the fifth character.
    let (sum, carry) = (at[4] as u32).overflowing_add(CLOCK_STEPS[4] as u32);
    Ok((carry, sum == 0))
}

/// The same with a font object in game memory, for Rust callers.
pub unsafe fn draw_clock_fixed_pitch_at(
    font: *mut Font,
    text: *mut u8,
    x: i32,
    y: i32,
    canvas: &mut dyn Canvas,
) -> Result<(bool, bool), Refused> {
    unsafe {
        let before = crate::pointers::GamePtr::new(crate::pointers::game_address(font))
            .read::<Font>("a font")?;
        let mut copy = before;
        let too_wide = draw_clock_fixed_pitch(&mut copy, text, x, y, canvas)?;
        if copy != before && canvas.keeps_font_changes() {
            crate::pointers::GamePtr::new(crate::pointers::game_address(font))
                .write(copy, "a font")?;
        }
        Ok(too_wide)
    }
}
