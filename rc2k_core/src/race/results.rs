//! The front end's tables the race takes in (from
//! `race::copy_league_table` 0x4650B7 and `race::copy_rally_times` 0x48C997): the league
//! table's ten records (0x50 bytes) turned into a row of their +8 and sixteen rows of
//! their sixteen dwords, and the rally's ten best times (records of 0x9C bytes) cut down
//! to the race's records of 0x50.

/// The records a league table or a rally's times hold.
pub const RECORDS: usize = 10;
/// The dwords each league record has from +0x10 (a row each in the race's table).
pub const LEAGUE_ROWS: usize = 16;

/// The league table in the race's layout: each record's +8 (`names`) and its
/// sixteen dwords from +0x10 as columns of sixteen rows (0x604134, 0x28 apart).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct League {
    pub firsts: [u32; RECORDS],
    pub rows: [[u32; RECORDS]; LEAGUE_ROWS],
}

/// `race::copy_league_table`: the table from its records (`record(n, offset)` the dword
/// at +offset of record n, asked in the original's order).
pub fn league(mut record: impl FnMut(usize, u32) -> u32) -> League {
    let mut l = League {
        firsts: [0; RECORDS],
        rows: [[0; RECORDS]; LEAGUE_ROWS],
    };
    for n in 0..RECORDS {
        l.firsts[n] = record(n, 8);
        for (k, row) in l.rows.iter_mut().enumerate() {
            row[n] = record(n, 0x10 + 4 * k as u32);
        }
    }
    l
}

/// A race's record of a best time: +0 .. +0xC from the rally's +0xC, +8, +0x10, +0x14,
/// then its sixteen dwords from +0x5C (the checkpoints' times).
pub type RaceTime = [u32; 20];

/// `race::copy_rally_times`: one record cut down (`field(offset)` the rally record's
/// dword, asked in the original's order).
pub fn race_time(mut field: impl FnMut(u32) -> u32) -> RaceTime {
    let mut t = [0u32; 20];
    t[3] = field(0x14);
    t[1] = field(8);
    t[2] = field(0x10);
    t[0] = field(0xC);
    for (k, v) in t[4..].iter_mut().enumerate() {
        *v = field(0x5C + 4 * k as u32);
    }
    t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn league_layout() {
        let l = league(|n, off| (n as u32) << 16 | off);
        assert_eq!(l.firsts[3], 3 << 16 | 8);
        assert_eq!(l.rows[0][9], 9 << 16 | 0x10);
        assert_eq!(l.rows[15][2], 2 << 16 | 0x4C);
    }

    #[test]
    fn times_cut_down() {
        let mut asked = Vec::new();
        let t = race_time(|off| {
            asked.push(off);
            off
        });
        assert_eq!(&t[..5], &[0xC, 8, 0x10, 0x14, 0x5C]);
        assert_eq!(t[19], 0x98);
        assert_eq!(&asked[..4], &[0x14, 8, 0x10, 0xC]);
    }
}
