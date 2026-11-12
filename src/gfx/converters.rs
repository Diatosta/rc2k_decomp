//! The MMX converters' masks and the converters table, out of RAL.EXE's image: IDA 0x4F51F0 ..
//! 0x4F52A4.

use crate::session::PlainDword;

/// How many masks (qwords) there are.
pub const MASKS: usize = 19;

/// The converters' masks and table.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Converters {
    /// `0x4F51F0` .. `0x4F5288`: the masks the MMX and 3DNow converters work with, 19
    /// qwords as (low, high) dwords, never written (the four formats' sets of five start
    /// at qwords 1, 5, 9 and 14 and share some: `mmx_converters`). The first qword is
    /// not used.
    pub masks: [PlainDword; 2 * MASKS],
    /// `dword_4F5288`: the converter (an address of the game's code) for each of the
    /// pixel formats' numbers, 0 for none (`install_converters`).
    pub table: [PlainDword; 7],
}

impl Converters {
    /// The masks and the table; the first values are the image's.
    pub const fn new() -> Self {
        Converters {
            masks: crate::gfx::plain_dwords([
                0xffff0000, 0xffff0000, 0x00ff00ff, 0x00ff00ff, 0xf800f800, 0xf800f800, 0x07e007e0,
                0x07e007e0, 0x001f001f, 0x001f001f, 0x00ff00ff, 0x00ff00ff, 0x7c007c00, 0x7c007c00,
                0x03e003e0, 0x03e003e0, 0x001f001f, 0x001f001f, 0x00ff00ff, 0x00ff00ff, 0x7c007c00,
                0x7c007c00, 0x03e003e0, 0x03e003e0, 0x001f001f, 0x001f001f, 0x80008000, 0x80008000,
                0x00ff00ff, 0x00ff00ff, 0x0f000f00, 0x0f000f00, 0x00f000f0, 0x00f000f0, 0x000f000f,
                0x000f000f, 0xf000f000, 0xf000f000,
            ]),
            table: [const { PlainDword::new(0) }; 7],
        }
    }
}

impl Converters {
    /// Mask number `n` (a qword).
    pub fn mask(&self, n: usize) -> u64 {
        u64::from(self.masks[2 * n].get()) | u64::from(self.masks[2 * n + 1].get()) << 32
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            &crate::gfx::owner::GFX.get().pixel.converters,
        );
    }
}
