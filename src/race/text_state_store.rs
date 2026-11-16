//! The frame text's state in the race block: `TextState.text_mode`, the debug drawing colour
//! number, and the two dwords after it (that colour as a 16-bit and as a 32-bit pixel,
//! `debug_pixel_16` and `debug_pixel_32`). `race::draw_frame_rate` sets the number to 2 each
//! frame; the debug overlay (`debug::ui`) reads none of the three, so they are written and never
//! read.

use crate::memory::cells::ZeroCells;

use crate::memory::session::PlainDword;

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct TextState {
    /// +0x0: the debug drawing colour number; `race::draw_frame_rate` sets it to 2 before the
    /// frame-rate text. Nothing reads it.
    pub text_mode: PlainDword,
    /// +0x4: the debug font's colour made a 16-bit pixel (`debug::draw`).
    pub debug_pixel_16: PlainDword,
    /// +0x8: the same, 32-bit.
    pub debug_pixel_32: PlainDword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl TextState {
    pub const fn new() -> Self {
        Self::ZERO
    }
}
