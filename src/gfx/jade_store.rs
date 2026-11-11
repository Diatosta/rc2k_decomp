//! The "jade" font's glyph table and colour (`gfx::load_jade`), out of RAL.EXE's image,
//! typed. Placed in a file of its own beside the graphics module's stores (converted in
//! parallel). `#[repr(C)]` struct with the image's layout, first values the image's (`tests`);
//! `moved::MOVED` points the image's addresses at it: the fonts' glyph-table pointers name it.

use crate::session::{Bytes, Dword, with_runs};

/// The jade font's glyphs, colour and file name (IDA 0x4F52C4 .. 0x4F66DD, the image's layout).
#[repr(C, align(16))]
#[derive(rc2k_derive::Cells)]
pub struct JadeFont {
    /// Not in the image: the bytes that keep every field the alignment its IDA address has (the struct is 16-byte aligned).
    pub _lead: Bytes<4>,
    /// 0x4F52C4: `unk_4F52C4`: the glyph table (`gfx::load_jade`).
    pub glyphs: Bytes<5120>,
    /// 0x4F66C4: `dword_4F66C4`: the text colour.
    pub colour: Dword,
    /// 0x4F66C8: `JADE`: the font's file name (`gfx::texts`).
    pub name: Bytes<21>,
    /// Up to the record's alignment (no byte of a record of cells outside a field).
    pub _tail: Bytes<3>,
}

impl JadeFont {
    /// The first values are the image's.
    pub const fn new() -> Self {
        JadeFont {
            _lead: Bytes::new([0; 4]),
            glyphs: Bytes::new([0; 5120]),
            colour: Dword::new(0x1234),
            name: Bytes::new(with_runs::<21>(&[(0, b"files\\d\\gfx\\jade.bun")])),
            _tail: Bytes::new([0; 3]),
        }
    }
}

impl JadeFont {
    pub const START: usize = 0x4F52C4;
    pub const LEN: usize = 0x1419;
    /// Where the byte IDA calls `START` is (`moved::MOVED`'s home).
    #[cfg(test)]
    pub fn home() -> usize {
        (crate::pointers::game_address(&raw const crate::gfx::owner::gfx().fonts.jade) + 4) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jadefont_starts_as_the_games() {
        let size = size_of::<JadeFont>();
        assert!(4 + JadeFont::LEN <= size && size <= (4 + JadeFont::LEN).next_multiple_of(16));
        let image = crate::moved::image_bytes(JadeFont::START, JadeFont::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(JadeFont::home()).cast_const(),
                JadeFont::LEN,
            )
        };
        assert_eq!(mine, &image[..]);
    }
}
