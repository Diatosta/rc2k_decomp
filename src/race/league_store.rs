//! The league's kept times and the automatic set-ups: the ten first records' ids, the sixteen rows
//! of ten kept times, the car's time at a split in race mode 3, the 36 set-ups of 0x10 bytes. The
//! fields keep a fixed layout.

use crate::memory::cells::ZeroCells;
use crate::memory::session::PlainDword;

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct League {
    /// The ten records' names (`race::copy_league_table`, `physics::compare_split_times`).
    pub names: [PlainDword; 10],
    /// Sixteen rows of ten kept times, 0x28 bytes each (`race::copy_league_table`;
    /// `physics::compare_split_times` reads up to split 27, rows past the sixteenth: a slip, kept).
    pub rows: [PlainDword; 160],
    /// Unused; it only keeps the layout.
    pub reserved_2a8: [PlainDword; 17],
    /// The car's time (`physics::compare_split_times`).
    pub car_time: PlainDword,
    /// The 36 automatic set-ups, 0x10 bytes each (h, s, g5, g4 + 1;
    /// `race::auto_set_up_car`).
    pub auto_setups: [PlainDword; 144],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece (a field of `race::owner::Race::course`), with its first values.
impl League {
    /// Kept time `n` of the rows. Splits 16 .. 27 read past the sixteen rows into the fields
    /// after them, in order.
    pub fn kept_time(&self, n: usize) -> Result<u32, crate::errors::BadInput> {
        if let Some(time) = self.rows.get(n) {
            return Ok(time.get());
        }
        crate::memory::session::read_runs(
            &[
                &self.rows,
                &self.reserved_2a8,
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
