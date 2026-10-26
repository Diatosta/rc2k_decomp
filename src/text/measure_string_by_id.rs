//! `text::measure_string_by_id` (0x40232C): builds the text of a string id in the shared text
//! buffer, NUL-terminates it and measures it with `measure_text`.
//!
//! In:  eax = string id, ebx = font (one in measure mode), ecx = x
//! Out: as `measure_text`: ebx = lines, ecx = the widest line, edx = y, eax = 0, carry;
//!      esi, edi, ebp preserved
//! 13 callers. A text longer than the buffer is refused: the original writes past it.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::gfx::blit_glyph::recording_works;
use crate::strings::string_table::{blob, is_table_id};
use crate::strings::text_from_ids::text;
use crate::text::draw_string_by_id::string_text;
use crate::text::draw_text::Canvas;
use crate::text::font::Font;
use crate::text::measure_text::{Measured, measure_text};
use std::ptr::copy_nonoverlapping;

// ---- Rust version ----

/// `buffer` is where the text is built: the game's text buffer, or a private one of at least
/// `TEXT_LEN` bytes. `font` is changed as the original changes the font object.
pub unsafe fn measure_string_by_id(
    id: u32,
    font: &mut Font,
    x: i32,
    buffer: *mut u8,
    canvas: &mut dyn Canvas,
) -> Result<Measured, Refused> {
    unsafe {
        let bytes = string_text(id)?;
        copy_nonoverlapping(bytes.as_ptr(), buffer, bytes.len());
        Ok(measure_text(font, buffer, x, canvas)?)
    }
}

/// The same with a font object in game memory and the game's text buffer, for Rust callers.
/// The font object is only written if the canvas keeps font changes.
pub unsafe fn measure_string_by_id_at(
    id: u32,
    font: *mut Font,
    x: i32,
    canvas: &mut dyn Canvas,
) -> Result<Measured, Refused> {
    if font.is_null() {
        return Err(BadInput("no font").into());
    }
    unsafe {
        let mut copy = font.read_unaligned();
        let measured = measure_string_by_id(id, &mut copy, x, text(), canvas)?;
        if canvas.keeps_font_changes() {
            font.write_unaligned(copy);
        }
        Ok(measured)
    }
}

pub const MEASURING_FONT: usize = 0x51E824; // dword_51E824: the font the name editors measure with
pub const TEST_STRINGS: [u32; 3] = [0x4E5, 0x4E6, 0x4E7];

/// Why a test that measures the game's strings cannot run, or None if it can.
pub fn measuring_unavailable() -> Option<&'static str> {
    if !recording_works() {
        return Some("the gfx::blit_glyph hook is off, so the original could really draw");
    }
    if blob().is_null() || !TEST_STRINGS.iter().all(|&id| is_table_id(id)) {
        return Some("the strings are not loaded");
    }
    (!crate::screens::font_is_loaded(MEASURING_FONT)).then_some("the measuring font is not loaded")
}
