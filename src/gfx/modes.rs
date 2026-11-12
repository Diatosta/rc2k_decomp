//! The screen mode tables (the pixel kinds' and thirds' bits, the pixel kinds, the sizes, the modes
//! listed), out of RAL.EXE's image: IDA 0x4F4E30 .. 0x4F503C. The thirds table (`thirds`) that
//! came before it in the image is `THIRDS`.
//!
//! The tables are walked by offset (`Modes::entries`: the entry whose first dword is
//! a key; a key not there gives the place past the table, as the original reads it), so
//! the struct keeps the image's layout and the tables' addresses come from it (`*_addr`).

use crate::gfx::plain_dwords;
use crate::session::PlainDword;
use core::mem::offset_of;

/// The modes the table holds (32 of 8 bytes).
pub const TABLE_ROOM: usize = 0x20;

/// The screen mode tables.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Modes {
    /// `dword_4F4E30`: the two pixel kinds' bits (the mode flags' 0x1000, 0x2000: the card).
    pub kind_bits: [PlainDword; 2],
    /// `dword_4F4E38`: the two thirds' bits.
    pub third_bits: [PlainDword; 2],
    /// `g_pixelKinds` (`dword_4F4E40`): two of (key, bits a pixel): (0x200, 16), (0x800, 32).
    pub pixel_kinds: [PlainDword; 4],
    /// `g_screenSizes` (`dword_4F4E50`): eight of (key, width and height as words).
    pub sizes: [PlainDword; 16],
    /// `dword_4F4E90`: three of (key, kind's number): the screen's kind (`g_screenKind`).
    pub kinds: [PlainDword; 6],
    /// `0x4F4EA8` .. `0x4F4F38`: more (key, figure) tables of pairs, a reading; nothing
    /// ported reads or writes them.
    pub other_tables: [PlainDword; 36],
    /// `g_screenModeCount` (`dword_4F4F38`): the modes in the table.
    pub count: PlainDword,
    /// `g_screenModeTable` (`dword_4F4F3C`): the modes, 8 bytes each (the mode, then its
    /// flags), room for 32.
    pub table: [PlainDword; 2 * TABLE_ROOM],
}

impl Modes {
    /// The tables; the first values are the image's.
    pub const fn new() -> Self {
        Modes {
            kind_bits: plain_dwords([0x1000, 0x2000]),
            third_bits: plain_dwords([0x20000, 0x40000]),
            pixel_kinds: plain_dwords([0x200, 0x10, 0x800, 0x20]),
            sizes: plain_dwords([
                0x1, 0x1e00280, 0x2, 0x2580320, 0x4, 0x2d003c0, 0x8, 0x3000400, 0x10, 0x3600480,
                0x20, 0x4000500, 0x40, 0x4b00640, 0x80, 0x5000780,
            ]),
            kinds: plain_dwords([0x10000, 0x0, 0x20000, 0x1, 0x40000, 0x2]),
            other_tables: plain_dwords([
                0x200, 0x10, 0x400, 0x18, 0x800, 0x20, 0x100, 0x8, 0x0, 0x0, 0x800, 0x20, 0x400,
                0x18, 0x200, 0x10, 0x100, 0x8, 0x0, 0x0, 0x400, 0x10, 0x200, 0x18, 0x100, 0x20,
                0x800, 0x8, 0x4000, 0x1, 0x2000, 0x2, 0x1000, 0x4, 0x0, 0x0,
            ]),
            count: PlainDword::new(0),
            table: [const { PlainDword::new(0) }; 2 * TABLE_ROOM],
        }
    }
}

impl Modes {
    /// The `count` entries of 8 bytes from byte `table` of the block (the tables are walked
    /// past their ends, inside the block); a read past the block ends the program.
    pub fn entries(
        &self,
        table: usize,
        count: u32,
    ) -> impl Iterator<Item = rc2k_core::system::tables::Entry> + '_ {
        (0..count as usize).map(move |n| {
            crate::cells::load_value(self, table.wrapping_add(8 * n))
                .unwrap_or_else(|e| crate::system::checked_reads::refuse(e))
        })
    }

    /// The second dword of the entry whose first dword is `key`, among the `count` entries
    /// from byte `table`; None when the key is not there.
    pub fn value_of(&self, key: u32, table: usize, count: u32) -> Option<u32> {
        let n = rc2k_core::system::tables::by_key(self.entries(table, count), key)?;
        Some(
            crate::cells::load_value(self, table + 8 * n + 4)
                .unwrap_or_else(|e| crate::fatal::raise::end("a screen mode table", e)),
        )
    }

    /// Whether one of the `count` modes listed has all of `bits`.
    pub fn lists_bits(&self, bits: u32) -> bool {
        rc2k_core::system::tables::with_bits(
            self.entries(offset_of!(Self, table), self.count.get()),
            bits,
        )
        .is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            &crate::gfx::owner::GFX.get().display.modes,
        );
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(offset_of!(Modes, pixel_kinds), 0x10);
        assert_eq!(offset_of!(Modes, sizes), 0x20);
        assert_eq!(offset_of!(Modes, kinds), 0x60);
        assert_eq!(offset_of!(Modes, count), 0x108);
        assert_eq!(offset_of!(Modes, table), 0x10c);
    }
}
