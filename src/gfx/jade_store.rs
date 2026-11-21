//! The "jade" font's glyph table and colour (`gfx::load_jade`) typed, by field.

use crate::memory::session::{PlainBytes, with_runs};
use std::cell::Cell;

/// The jade font's glyphs, colour and file name.
pub struct JadeFont {
    /// The glyph table (`gfx::load_jade`).
    pub glyphs: PlainBytes<5120>,
    /// The text colour.
    pub colour: Cell<u32>,
    /// The font's file name (`gfx::texts`).
    pub name: PlainBytes<21>,
}

impl JadeFont {
    /// The font's fields.
    pub const fn new() -> Self {
        JadeFont {
            glyphs: PlainBytes::new([0; 5120]),
            colour: Cell::new(0x1234),
            name: PlainBytes::new(with_runs::<21>(&[(0, b"files\\d\\gfx\\jade.bun")])),
        }
    }
}
