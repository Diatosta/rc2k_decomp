//! The league's kept times and the automatic set-ups (IDA 0x60410C .. 0x60463C; phase 7,
//! typed): the ten first records' ids, the sixteen rows of ten kept times, the car's time at
//! a split in race mode 3, the 36 set-ups of 0x10 bytes. The fields are in the image's order. Names are readings unless IDA's.

use crate::cells::ZeroCells;
use crate::session::PlainDword;

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
    /// Kept time `n` of the rows. The original does not hold `n` to the sixteen rows (the
    /// splits 16 .. 27 read past them): past the rows it reads the fields after them, in order.
    pub fn kept_time(&self, n: usize) -> Result<u32, crate::errors::BadInput> {
        if let Some(time) = self.rows.get(n) {
            return Ok(time.get());
        }
        crate::session::read_runs(
            &[
                &self.rows,
                &self.unused_2a8,
                &self.car_time,
                &self.auto_setups,
            ],
            n,
            "a kept split time",
        )
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    /// The rows are read by index, and an index far past the record is refused.
    #[test]
    fn kept_time_reads_the_rows_and_refuses_far_past_them() {
        let league = League::new();
        for (n, row) in league.rows.iter().enumerate() {
            row.set(0x1000 + n as u32);
            assert_eq!(
                league.kept_time(n).ok(),
                Some(0x1000 + n as u32),
                "index {n}"
            );
        }
        assert!(league.kept_time(1 << 20).is_err());
    }
}
