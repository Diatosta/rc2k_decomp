//! Functions that lay out parts of a screen: fixed positions, fonts and string ids.

use crate::framework::{GameHook, game_ptr, rva};
use crate::text::font::Font;
use std::sync::atomic::{AtomicBool, Ordering};

pub mod draw_stage_heading;
pub mod draw_stage_info;
pub mod draw_stage_records;

pub static HOOKS: &[&dyn GameHook] = &[
    &draw_stage_heading::HOOK,
    &draw_stage_records::HOOK,
    &draw_stage_info::HOOK,
];

/// Console switch `blank on|off`, kept for every hook in this module: while set, the hooks of this module return at once,
/// running neither the original nor the Rust version, so whatever they draw disappears from
/// the screen. That is how to confirm, in the game, what a screen function draws. It is
/// harmless for these functions because they preserve every register; a new hook in this
/// module must check `blanked()` right after `count_call()`.
pub static BLANK: AtomicBool = AtomicBool::new(false);

pub fn blanked() -> bool {
    BLANK.load(Ordering::Relaxed)
}

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
