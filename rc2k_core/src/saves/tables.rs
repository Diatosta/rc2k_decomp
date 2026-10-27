//! The best times' tables in memory (converted 2026-10-21 from
//! `saves::allocate_best_times` 0x47C4D8 and `saves::count_stages` 0x47C95D, a plain
//! function there).
//!
//! Nine tables are taken once, their sizes fixed but for two that hold an entry of 0x618
//! bytes for each stage of the championship's record table (`g_recordTable`, records of
//! 0xA4 bytes ending with a -1, a stage's tag 0). The sixth and the seventh end with -1.

use crate::Invalid;

pub const TABLES: usize = 9;
/// A stage's entry in the two tables of stages.
pub const STAGE_ENTRY_LEN: u32 = 0x618;
/// The tables that end with -1 in their last dword.
pub const MARKED: [usize; 2] = [5, 6];
/// More records than any table has.
pub const MOST_RECORDS: u32 = 0x1000;

/// A typed table as `var\hst.dat` holds it: exactly `LEN` bytes, in the original's layout.
pub trait FileTable: Default {
    const LEN: usize;
    /// From its bytes (`b` at least `LEN` long).
    fn read(b: &[u8]) -> Self;
    /// Into its `LEN` bytes.
    fn write(&self) -> Vec<u8>;
}

/// `saves::count_stages`: the records with tag 0, up to the table's -1 (`tag` record n's
/// first dword; the first record is not looked at for the end). Refused: no end within
/// `MOST_RECORDS`.
pub fn count_stages(tag: impl Fn(u32) -> u32) -> Result<u32, Invalid> {
    let mut stages = 0;
    for n in 0..MOST_RECORDS {
        if tag(n) == 0 {
            stages += 1;
        }
        if tag(n + 1) == u32::MAX {
            return Ok(stages);
        }
    }
    Err(Invalid("a record table without its end"))
}

/// The nine tables' sizes for a championship of that many stages.
pub fn sizes(stages: u32) -> [u32; TABLES] {
    let entries = stages.wrapping_mul(STAGE_ENTRY_LEN);
    [
        0x1C20, 0xBB8, entries, 0x15E0, 0xDB60, 0x4C2C4, 0x3A984, 0xDB60, entries,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stages_and_sizes() {
        let tags = [5u32, 0, 0, 3, 0, u32::MAX];
        assert_eq!(count_stages(|n| tags[n as usize]), Ok(3));
        // the first record's -1 is not an end
        let tags = [u32::MAX, 0, u32::MAX];
        assert_eq!(count_stages(|n| tags[n as usize]), Ok(1));
        assert!(count_stages(|_| 0).is_err());
        assert_eq!(sizes(2)[2], 0xC30);
        assert_eq!(sizes(2)[8], 0xC30);
    }
}
