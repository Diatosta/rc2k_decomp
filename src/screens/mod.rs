//! Functions that lay out parts of a screen: fixed positions, fonts and string ids.

use crate::framework::{GameHook, game_ptr, rva};
use crate::text::font::Font;

pub mod draw_record_rows;
pub mod draw_stage_heading;

pub static HOOKS: &[&dyn GameHook] = &[&draw_stage_heading::HOOK, &draw_record_rows::HOOK];

/// byte_A8BB54: the text buffer these functions build their lines in.
pub const LINE_BUFFER_RVA: usize = rva(0xA8BB54);

pub fn line_buffer() -> *mut u8 {
    game_ptr(LINE_BUFFER_RVA)
}

/// A font object in game memory, by its address in IDA.
pub fn font_at(addr: usize) -> *mut Font {
    game_ptr(rva(addr))
}

/// Copies `bytes` (a text with its NUL) to the line buffer.
pub unsafe fn set_line(bytes: &[u8]) {
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), line_buffer(), bytes.len()) };
}

/// True if the font object at `addr` has a glyph table, so that text can be measured with it.
pub fn font_is_loaded(addr: usize) -> bool {
    unsafe { font_at(addr).read_unaligned() }.glyph_table != 0
}
