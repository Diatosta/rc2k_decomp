//! The league's kept times and the automatic set-ups (IDA 0x60410C .. 0x60463C; phase 7,
//! typed): the ten first records' ids, the sixteen rows of ten kept times, the car's time at
//! a split in race mode 3, the 36 set-ups of 0x10 bytes. `#[repr(C)]` with the image's
//! layout, over the race block's bytes (`LEAGUE`). Names are readings unless IDA's.

use crate::session::Dword;

#[repr(C)]
#[derive(rc2k_derive::Cells)]
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

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece: a static of its own since 2026-10-29 (it left the race block, phase 13:
/// `race::store::LEFT`), with the image's first values.
pub static LEAGUE: League =
    crate::pointers::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START));

/// IDA 0x60410C, the first byte of the piece.
pub const START: usize = 0x60410C;
/// IDA 0x60463C, the end of the piece.
pub const END: usize = 0x60463C;

const _: () = assert!(END - START == core::mem::size_of::<League>());

/// Where IDA 0x60410C is now.
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(core::ptr::from_ref(&LEAGUE))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&LEAGUE, START, END);
    }

    #[test]
    fn piece_has_its_home() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            crate::moved::place(END - 1),
            (piece_addr() + (END - START) as u32 - 1) as usize
        );
        assert_eq!(offset_of!(League, rows), 0x604134 - START);
        assert_eq!(offset_of!(League, car_time), 0x6043F8 - START);
        assert_eq!(offset_of!(League, auto_setups), 0x6043FC - START);
    }
}
