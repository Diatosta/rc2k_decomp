//! The screen mode tables (the pixel kinds' and thirds' bits, the pixel kinds, the sizes, the modes
//! listed), out of RAL.EXE's image: IDA 0x4F4E30 .. 0x4F503C. The thirds table (`byte_4F4B30`) that
//! came before it in the image is `THIRDS`.
//!
//! The tables are walked by address (`system::find_entry`: the entry whose first dword is
//! a key; a key not there gives the place past the table, as the original reads it), so
//! the struct keeps the image's layout and the tables' addresses come from it (`*_addr`).

use crate::gfx::dwords;
use crate::session::Dword;

/// IDA's address of the block's first byte.
pub const START: usize = 0x4F4E30;
/// The block's length.
pub const LEN: usize = 0x4F503C - START;

/// The modes the table holds (32 of 8 bytes).
pub const TABLE_ROOM: usize = 0x20;

/// The screen mode tables.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Modes {
    /// `dword_4F4E30`: the two pixel kinds' bits (the mode flags' 0x1000, 0x2000: the card).
    pub kind_bits: [Dword; 2],
    /// `dword_4F4E38`: the two thirds' bits.
    pub third_bits: [Dword; 2],
    /// `g_pixelKinds` (`dword_4F4E40`): two of (key, bits a pixel): (0x200, 16), (0x800, 32).
    pub pixel_kinds: [Dword; 4],
    /// `g_screenSizes` (`dword_4F4E50`): eight of (key, width and height as words).
    pub sizes: [Dword; 16],
    /// `dword_4F4E90`: three of (key, kind's number): the screen's kind (`g_screenKind`).
    pub kinds: [Dword; 6],
    /// `0x4F4EA8` .. `0x4F4F38`: more (key, figure) tables of pairs, a reading; nothing
    /// ported reads or writes them.
    pub other_tables: [Dword; 36],
    /// `g_screenModeCount` (`dword_4F4F38`): the modes in the table.
    pub count: Dword,
    /// `g_screenModeTable` (`dword_4F4F3C`): the modes, 8 bytes each (the mode, then its
    /// flags), room for 32.
    pub table: [Dword; 2 * TABLE_ROOM],
}

impl Modes {
    /// The tables; the first values are the image's.
    pub const fn new() -> Self {
        Modes {
            kind_bits: dwords([0x1000, 0x2000]),
            third_bits: dwords([0x20000, 0x40000]),
            pixel_kinds: dwords([0x200, 0x10, 0x800, 0x20]),
            sizes: dwords([
                0x1, 0x1e00280, 0x2, 0x2580320, 0x4, 0x2d003c0, 0x8, 0x3000400, 0x10, 0x3600480,
                0x20, 0x4000500, 0x40, 0x4b00640, 0x80, 0x5000780,
            ]),
            kinds: dwords([0x10000, 0x0, 0x20000, 0x1, 0x40000, 0x2]),
            other_tables: dwords([
                0x200, 0x10, 0x400, 0x18, 0x800, 0x20, 0x100, 0x8, 0x0, 0x0, 0x800, 0x20, 0x400,
                0x18, 0x200, 0x10, 0x100, 0x8, 0x0, 0x0, 0x400, 0x10, 0x200, 0x18, 0x100, 0x20,
                0x800, 0x8, 0x4000, 0x1, 0x2000, 0x2, 0x1000, 0x4, 0x0, 0x0,
            ]),
            count: Dword::new(0),
            table: [const { Dword::new(0) }; 2 * TABLE_ROOM],
        }
    }
}

#[cfg(test)]
/// Where the block is (`moved::MOVED`'s home for IDA 0x4F4E30).
pub fn home() -> usize {
    crate::pointers::game_address(&raw const crate::gfx::owner::GFX.display.modes) as usize
}

impl Modes {
    /// Where a table is, as a pointer bounded by the whole block (the tables are walked
    /// past their ends).
    fn table_ptr(offset: usize) -> crate::pointers::GamePtr {
        crate::pointers::GamePtr::to(&crate::gfx::owner::GFX.display.modes).offset(offset as u32)
    }
    pub fn pixel_kinds_ptr() -> crate::pointers::GamePtr {
        Self::table_ptr(core::mem::offset_of!(Self, pixel_kinds))
    }
    pub fn sizes_ptr() -> crate::pointers::GamePtr {
        Self::table_ptr(core::mem::offset_of!(Self, sizes))
    }
    pub fn kinds_ptr() -> crate::pointers::GamePtr {
        Self::table_ptr(core::mem::offset_of!(Self, kinds))
    }
    /// The mode table (walked past its end like the others).
    pub fn mode_table_ptr() -> crate::pointers::GamePtr {
        Self::table_ptr(core::mem::offset_of!(Self, table))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(
            &crate::gfx::owner::GFX.display.modes,
            START,
            START + LEN,
        );
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(offset_of!(Modes, pixel_kinds), 0x4F4E40 - START);
        assert_eq!(offset_of!(Modes, sizes), 0x4F4E50 - START);
        assert_eq!(offset_of!(Modes, kinds), 0x4F4E90 - START);
        assert_eq!(offset_of!(Modes, count), 0x4F4F38 - START);
        assert_eq!(offset_of!(Modes, table), 0x4F4F3C - START);
        assert_eq!(crate::moved::place(START), home());
        assert_eq!(
            crate::moved::place(0x4F4E50),
            Modes::sizes_ptr().addr() as usize
        );
        assert_eq!(
            crate::moved::place(0x4F4F3C),
            Modes::mode_table_ptr().addr() as usize
        );
    }
}
