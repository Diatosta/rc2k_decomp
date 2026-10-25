//! The league's kept times and the automatic set-ups (IDA 0x60410C .. 0x60463C; phase 7,
//! typed): the ten first records' ids, the sixteen rows of ten kept times, the car's time at
//! a split in race mode 3, the 36 set-ups of 0x10 bytes. `#[repr(C)]` with the image's
//! layout, over the race block's bytes (`LEAGUE`). Names are readings unless IDA's.

use crate::session::Dword;

#[repr(C)]
pub struct League {
    /// +0x0 (0x60410C): `dword_60410C`: the ten records' names (`race::copy_league_table`, `physics::compare_split_times`).
    pub names: [Dword; 10],
    /// +0x28 (0x604134): `unk_604134`: sixteen rows of ten kept times, 0x28 bytes each (`race::copy_league_table`; `physics::compare_split_times` reads up to split 27, rows past the sixteenth: a slip, kept).
    pub rows: [Dword; 160],
    /// +0x2a8 (0x6043B4): not named.
    pub unknown_2a8: [Dword; 17],
    /// +0x2ec (0x6043F8): `dword_6043F8`: the car's time (`physics::compare_split_times`).
    pub car_time: Dword,
    /// +0x2f0 (0x6043FC): `g_autoSetups`: the 36 automatic set-ups, 0x10 bytes each (h, s, g5, g4 + 1; `race::auto_set_up_car`).
    pub auto_setups: [Dword; 144],
}

/// The piece: the race block's bytes at IDA 0x60410C seen as `League`.
pub struct InBlock;

impl core::ops::Deref for InBlock {
    type Target = League;
    fn deref(&self) -> &League {
        // SAFETY: the race block lives as long as the program and is aligned to 4 (the
        // test checks); every field is an atomic, as the block's bytes were shared.
        unsafe { &*(piece_addr() as *const League) }
    }
}

pub static LEAGUE: InBlock = InBlock;

/// IDA 0x60410C, the first byte of the piece.
pub const START: usize = 0x60410C;
/// IDA 0x60463C, the end of the piece.
pub const END: usize = 0x60463C;

const _: () = assert!(END - START == core::mem::size_of::<League>());

/// Where IDA 0x60410C is now.
pub fn piece_addr() -> usize {
    crate::moved::place(START)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn piece_sits_on_the_block() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            crate::moved::place(END - 1),
            piece_addr() + (END - START) - 1
        );
        assert_eq!(offset_of!(League, rows), 0x604134 - START);
        assert_eq!(offset_of!(League, car_time), 0x6043F8 - START);
        assert_eq!(offset_of!(League, auto_setups), 0x6043FC - START);
    }
}
