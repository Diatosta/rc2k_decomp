//! The front-end tables the race takes in: the league table's ten 0x50-byte records as a row
//! of their third dword and sixteen rows of their later dwords, and the rally's ten best times
//! (0x9C-byte records) cut to 0x50-byte race records.

/// Records in a league table or a rally's times.
pub const RECORDS: usize = 10;
/// Dwords per league record from its fifth on.
pub(crate) const LEAGUE_ROWS: usize = 16;

/// The league table in race layout: each record's third dword (`firsts`) and its dwords from the
/// fifth on as columns of sixteen rows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct League {
    pub firsts: [u32; RECORDS],
    pub rows: [[u32; RECORDS]; LEAGUE_ROWS],
}

/// Builds the table; `record(n, offset)` reads a dword of record n, asked in a fixed order.
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

/// A race's best-time record: the rally record's fourth, third, fifth and sixth dwords, then
/// sixteen dwords from its 24th (checkpoint times).
pub(crate) type RaceTime = [u32; 20];

/// Cuts one record down; `field(offset)` reads the rally record, asked in a fixed order.
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
