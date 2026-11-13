//! The screen mode tables (the pixel kinds' and thirds' bits, the pixel kinds, the sizes, the modes
//! listed), out of RAL.EXE's image: IDA 0x4F4E30 .. 0x4F503C. The thirds table (`thirds`) that
//! came before it in the image is `THIRDS`.
//!
//! The tables are walked by dword (`Modes::entries`: the entry whose first dword is
//! a key; a key not there gives the place past the table, as the original reads it), through
//! the fields in the record's order (`Modes::dword`).

use crate::errors::BadInput;
use crate::gfx::plain_dwords;
use crate::session::{PlainDword, Run};
#[cfg(test)]
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

/// Where the tables start, counted in dwords from the first field.
impl Modes {
    pub const PIXEL_KINDS_AT: usize = 4;
    pub const SIZES_AT: usize = 8;
    pub const KINDS_AT: usize = 24;
    pub const TABLE_AT: usize = 67;
}

impl Modes {
    /// The fields in the order of the record, each a run of dwords.
    fn runs(&self) -> [&dyn Run; 8] {
        [
            &self.kind_bits,
            &self.third_bits,
            &self.pixel_kinds,
            &self.sizes,
            &self.kinds,
            &self.other_tables,
            &self.count,
            &self.table,
        ]
    }

    /// Dword `n` of the record. A table that is walked to the key it does not hold (`kinds`)
    /// goes on into the field after it; past the last field is refused.
    pub fn dword(&self, n: usize) -> Result<u32, BadInput> {
        crate::session::read_runs(&self.runs(), n, "a screen mode table's dword")
    }

    /// The `count` entries of two dwords from dword `at` of the record (the tables are walked
    /// past their ends, inside the record); a read past the record ends the program.
    pub fn entries(
        &self,
        at: usize,
        count: u32,
    ) -> impl Iterator<Item = rc2k_core::system::tables::Entry> + '_ {
        (0..count as usize).map(move |n| {
            let entry = |k: usize| {
                self.dword(at.wrapping_add(2 * n + k))
                    .unwrap_or_else(|e| crate::system::checked_reads::refuse(e))
            };
            [entry(0), entry(1)]
        })
    }

    /// The second dword of the entry whose first dword is `key`, among the `count` entries
    /// from dword `at`; None when the key is not there.
    pub fn value_of(&self, key: u32, at: usize, count: u32) -> Option<u32> {
        let n = rc2k_core::system::tables::by_key(self.entries(at, count), key)?;
        Some(
            self.dword(at + 2 * n + 1)
                .unwrap_or_else(|e| crate::fatal::raise::end("a screen mode table", e)),
        )
    }

    /// Whether one of the `count` modes listed has all of `bits`.
    pub fn lists_bits(&self, bits: u32) -> bool {
        rc2k_core::system::tables::with_bits(self.entries(Self::TABLE_AT, self.count.get()), bits)
            .is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The tables' places in dwords are the fields' places in the record.
    #[test]
    fn places_are_the_fields() {
        assert_eq!(Modes::PIXEL_KINDS_AT * 4, offset_of!(Modes, pixel_kinds));
        assert_eq!(Modes::SIZES_AT * 4, offset_of!(Modes, sizes));
        assert_eq!(Modes::KINDS_AT * 4, offset_of!(Modes, kinds));
        assert_eq!(Modes::TABLE_AT * 4, offset_of!(Modes, table));
    }

    /// The old byte read of the record, kept to check the reads by field.
    fn old_dword(m: &Modes, at: usize) -> Result<u32, BadInput> {
        crate::cells::load_value(m, at)
    }

    /// Every dword to the end of the record reads as the byte read did; the first past it is
    /// refused by both.
    #[test]
    fn dwords_read_as_the_byte_read() {
        let m = Modes::new();
        for (n, d) in m.table.iter().enumerate() {
            d.set(0x1000_0001 + n as u32);
        }
        m.count.set(7);
        let total = size_of::<Modes>() / 4;
        for n in 0..total + 4 {
            assert_eq!(m.dword(n).ok(), old_dword(&m, 4 * n).ok(), "dword {n}");
        }
        assert!(m.dword(total).is_err());
    }
}
