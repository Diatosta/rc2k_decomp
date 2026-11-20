//! The league's kept times and the automatic set-ups: the ten first records' ids, the sixteen rows
//! of ten kept times, the car's time at a split in race mode 3, the 36 set-ups of 0x10 bytes.

use crate::errors::BadInput;

pub struct League {
    /// The ten records' names (`race::copy_league_table`, `physics::compare_split_times`).
    pub names: [u32; 10],
    /// Sixteen rows of ten kept times (`race::copy_league_table`;
    /// `physics::compare_split_times` reads up to split 27, rows past the sixteenth: a slip, kept).
    pub rows: [u32; 160],
    /// Unused; the kept-time reads past the rows run through it.
    pub reserved: [u32; 17],
    /// The car's time (`physics::compare_split_times`).
    pub car_time: u32,
    /// The 36 automatic set-ups, 4 values each (h, s, g5, g4 + 1; `race::auto_set_up_car`).
    pub auto_setups: [u32; 144],
}

impl League {
    /// Kept time `n` of the rows. Splits 16 .. 27 read past the sixteen rows into the fields
    /// after them, in order (the rows, the reserved values, the car's time, the set-ups).
    pub fn kept_time(&self, n: usize) -> Result<u32, BadInput> {
        let past_rows = n.wrapping_sub(self.rows.len());
        if let Some(time) = self.rows.get(n) {
            return Ok(*time);
        }
        if let Some(value) = self.reserved.get(past_rows) {
            return Ok(*value);
        }
        let past_reserved = past_rows.wrapping_sub(self.reserved.len());
        if past_reserved == 0 {
            return Ok(self.car_time);
        }
        self.auto_setups
            .get(past_reserved.wrapping_sub(1))
            .copied()
            .ok_or(BadInput("a kept split time"))
    }

    pub const fn new() -> Self {
        League {
            names: [0; 10],
            rows: [0; 160],
            reserved: [0; 17],
            car_time: 0,
            auto_setups: [
                0x4C, 0x80, 1, 4, 0x4C, 0x80, 1, 3, 0x4C, 0x80, 1, 3, 0x4C, 0x80, 1, 2, 0x4C, 0x80,
                1, 3, 0x4C, 0x80, 2, 3, 0xB4, 0x4C, 3, 1, 0xB4, 0x4C, 3, 2, 0xB4, 0x4C, 3, 2, 0xB4,
                0x4C, 3, 2, 0xB4, 0x4C, 3, 2, 0xB4, 0x4C, 4, 1, 0xB4, 0x4C, 3, 1, 0xB4, 0x4C, 3, 1,
                0xB4, 0x4C, 3, 2, 0xB4, 0x4C, 5, 1, 0xB4, 0x4C, 5, 1, 0xB4, 0x4C, 5, 1, 0x4C, 0x80,
                2, 2, 0x4C, 0x80, 1, 2, 0x4C, 0x80, 1, 2, 0x4C, 0x80, 1, 2, 0x4C, 0x80, 1, 2, 0x4C,
                0x80, 1, 2, 0xB4, 0x4C, 3, 1, 0xB4, 0x4C, 3, 1, 0xB4, 0x4C, 3, 2, 0xB4, 0x4C, 3, 1,
                0xB4, 0x4C, 4, 1, 0xB4, 0x4C, 4, 2, 0x4C, 0x80, 1, 2, 0x4C, 0x80, 1, 2, 0x4C, 0x80,
                1, 4, 0x4C, 0x80, 1, 2, 0x4C, 0x80, 1, 2, 0x4C, 0x80, 1, 3,
            ],
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
        let mut league = league;
        for n in 0..league.rows.len() {
            league.rows[n] = 0x1000 + n as u32;
            assert_eq!(
                league.kept_time(n).ok(),
                Some(0x1000 + n as u32),
                "index {n}"
            );
        }
        assert!(league.kept_time(1 << 20).is_err());
    }
}
