//! The "jade" font's glyph table and colour (`gfx::load_jade`), out of RAL.EXE's image,
//! typed, by field. Placed in a file of its own beside the graphics module's stores (converted in
//! parallel). First values the image's (`tests`).

use crate::session::{PlainBytes, PlainDword, with_runs};

/// The jade font's glyphs, colour and file name (IDA 0x4F52C4 .. 0x4F66DD).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct JadeFont {
    /// 0x4F52C4: `unk_4F52C4`: the glyph table (`gfx::load_jade`).
    pub glyphs: PlainBytes<5120>,
    /// 0x4F66C4: `dword_4F66C4`: the text colour.
    pub colour: PlainDword,
    /// 0x4F66C8: `JADE`: the font's file name (`gfx::texts`).
    pub name: PlainBytes<21>,
    /// Up to the record's alignment: the derive takes no padding after the last field.
    pub _tail: PlainBytes<3>,
}

impl JadeFont {
    /// The first values are the image's.
    pub const fn new() -> Self {
        JadeFont {
            glyphs: PlainBytes::new([0; 5120]),
            colour: PlainDword::new(0x1234),
            name: PlainBytes::new(with_runs::<21>(&[(0, b"files\\d\\gfx\\jade.bun")])),
            _tail: PlainBytes::new([0; 3]),
        }
    }
}
