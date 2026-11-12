//! The league's kept times and the automatic set-ups (IDA 0x60410C .. 0x60463C; phase 7,
//! typed): the ten first records' ids, the sixteen rows of ten kept times, the car's time at
//! a split in race mode 3, the 36 set-ups of 0x10 bytes. `#[repr(C)]` with the image's
//! layout, over the race block's bytes (`LEAGUE`). Names are readings unless IDA's.

use crate::cells::ZeroCells;
use crate::session::PlainDword;

#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct League {
    /// +0x0 (0x60410C): `dword_60410C`: the ten records' names (`race::copy_league_table`, `physics::compare_split_times`).
    pub names: [PlainDword; 10],
    /// +0x28 (0x604134): `unk_604134`: sixteen rows of ten kept times, 0x28 bytes each (`race::copy_league_table`; `physics::compare_split_times` reads up to split 27, rows past the sixteenth: a slip, kept).
    pub rows: [PlainDword; 160],
    /// +0x2a8 (0x6043B4): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_2a8: [PlainDword; 17],
    /// +0x2ec (0x6043F8): `dword_6043F8`: the car's time (`physics::compare_split_times`).
    pub car_time: PlainDword,
    /// +0x2f0 (0x6043FC): `g_autoSetups`: the 36 automatic set-ups, 0x10 bytes each (h, s, g5, g4 + 1; `race::auto_set_up_car`).
    pub auto_setups: [PlainDword; 144],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece (a field of `race::owner::Race::course`), with the image's first values.
impl League {
    pub const fn new() -> Self {
        Self {
            auto_setups: [
                PlainDword::new(0x4C),
                PlainDword::new(0x80),
                PlainDword::new(1),
                PlainDword::new(4),
                PlainDword::new(0x4C),
                PlainDword::new(0x80),
                PlainDword::new(1),
                PlainDword::new(3),
                PlainDword::new(0x4C),
                PlainDword::new(0x80),
                PlainDword::new(1),
                PlainDword::new(3),
                PlainDword::new(0x4C),
                PlainDword::new(0x80),
                PlainDword::new(1),
                PlainDword::new(2),
                PlainDword::new(0x4C),
                PlainDword::new(0x80),
                PlainDword::new(1),
                PlainDword::new(3),
                PlainDword::new(0x4C),
                PlainDword::new(0x80),
                PlainDword::new(2),
                PlainDword::new(3),
                PlainDword::new(0xB4),
                PlainDword::new(0x4C),
                PlainDword::new(3),
                PlainDword::new(1),
                PlainDword::new(0xB4),
                PlainDword::new(0x4C),
                PlainDword::new(3),
                PlainDword::new(2),
                PlainDword::new(0xB4),
                PlainDword::new(0x4C),
                PlainDword::new(3),
                PlainDword::new(2),
                PlainDword::new(0xB4),
                PlainDword::new(0x4C),
                PlainDword::new(3),
                PlainDword::new(2),
                PlainDword::new(0xB4),
                PlainDword::new(0x4C),
                PlainDword::new(3),
                PlainDword::new(2),
                PlainDword::new(0xB4),
                PlainDword::new(0x4C),
                PlainDword::new(4),
                PlainDword::new(1),
                PlainDword::new(0xB4),
                PlainDword::new(0x4C),
                PlainDword::new(3),
                PlainDword::new(1),
                PlainDword::new(0xB4),
                PlainDword::new(0x4C),
                PlainDword::new(3),
                PlainDword::new(1),
                PlainDword::new(0xB4),
                PlainDword::new(0x4C),
                PlainDword::new(3),
                PlainDword::new(2),
                PlainDword::new(0xB4),
                PlainDword::new(0x4C),
                PlainDword::new(5),
                PlainDword::new(1),
                PlainDword::new(0xB4),
                PlainDword::new(0x4C),
                PlainDword::new(5),
                PlainDword::new(1),
                PlainDword::new(0xB4),
                PlainDword::new(0x4C),
                PlainDword::new(5),
                PlainDword::new(1),
                PlainDword::new(0x4C),
                PlainDword::new(0x80),
                PlainDword::new(2),
                PlainDword::new(2),
                PlainDword::new(0x4C),
                PlainDword::new(0x80),
                PlainDword::new(1),
                PlainDword::new(2),
                PlainDword::new(0x4C),
                PlainDword::new(0x80),
                PlainDword::new(1),
                PlainDword::new(2),
                PlainDword::new(0x4C),
                PlainDword::new(0x80),
                PlainDword::new(1),
                PlainDword::new(2),
                PlainDword::new(0x4C),
                PlainDword::new(0x80),
                PlainDword::new(1),
                PlainDword::new(2),
                PlainDword::new(0x4C),
                PlainDword::new(0x80),
                PlainDword::new(1),
                PlainDword::new(2),
                PlainDword::new(0xB4),
                PlainDword::new(0x4C),
                PlainDword::new(3),
                PlainDword::new(1),
                PlainDword::new(0xB4),
                PlainDword::new(0x4C),
                PlainDword::new(3),
                PlainDword::new(1),
                PlainDword::new(0xB4),
                PlainDword::new(0x4C),
                PlainDword::new(3),
                PlainDword::new(2),
                PlainDword::new(0xB4),
                PlainDword::new(0x4C),
                PlainDword::new(3),
                PlainDword::new(1),
                PlainDword::new(0xB4),
                PlainDword::new(0x4C),
                PlainDword::new(4),
                PlainDword::new(1),
                PlainDword::new(0xB4),
                PlainDword::new(0x4C),
                PlainDword::new(4),
                PlainDword::new(2),
                PlainDword::new(0x4C),
                PlainDword::new(0x80),
                PlainDword::new(1),
                PlainDword::new(2),
                PlainDword::new(0x4C),
                PlainDword::new(0x80),
                PlainDword::new(1),
                PlainDword::new(2),
                PlainDword::new(0x4C),
                PlainDword::new(0x80),
                PlainDword::new(1),
                PlainDword::new(4),
                PlainDword::new(0x4C),
                PlainDword::new(0x80),
                PlainDword::new(1),
                PlainDword::new(2),
                PlainDword::new(0x4C),
                PlainDword::new(0x80),
                PlainDword::new(1),
                PlainDword::new(2),
                PlainDword::new(0x4C),
                PlainDword::new(0x80),
                PlainDword::new(1),
                PlainDword::new(3),
            ],
            ..ZeroCells::ZERO
        }
    }
}

/// IDA 0x60410C, the first byte of the piece.
pub const START: usize = 0x60410C;
/// IDA 0x60463C, the end of the piece.
pub const END: usize = 0x60463C;

const _: () = assert!(END - START == core::mem::size_of::<League>());

#[cfg(test)]
mod tests {

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            crate::race::owner::RACE.course.league(),
        );
    }
}
