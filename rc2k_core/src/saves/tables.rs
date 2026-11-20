//! The best times' tables in memory: nine tables taken once, their sizes fixed but for two that
//! hold 0x618 bytes for each stage of the championship's record table (records of 0xA4 bytes
//! ending with a -1, a stage's tag 0). The sixth and the seventh end with -1.

use crate::Invalid;

pub const TABLES: usize = 9;
/// A stage's entry in the two tables of stages, in bytes.
pub(crate) const STAGE_ENTRY_LEN: u32 = 0x618;
/// More records than any table has.
pub const MOST_RECORDS: u32 = 0x1000;

/// A typed table as `var\hst.dat` holds it.
pub trait FileTable: Sized {
    /// A new one (zeros) of the size `allocate_best_times` asks; None for an impossible size.
    fn zeroed(bytes: usize) -> Option<Self>;
    /// From its bytes (as many as `write` gives).
    fn read(b: &[u8]) -> Self;
    /// Into its bytes.
    fn write(&self) -> Vec<u8>;
}

/// The records with tag 0, up to the table's -1 (`tag` is record n's first dword; the first
/// record is not looked at for the end). Refused: no end within `MOST_RECORDS`.
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

/// The six tables of `var\hst.dat`, each made from varied bytes and written out again, are
/// the bytes they were when first kept (a checksum of each).
#[cfg(test)]
mod golden {
    use super::*;
    use crate::saves::entries::{LevelScores, StageTimes};
    use crate::saves::scores::BestScores;

    fn checksum(bytes: &[u8]) -> u64 {
        bytes.iter().fold(0xcbf2_9ce4_8422_2325_u64, |h, &b| {
            (h ^ u64::from(b)).wrapping_mul(0x100_0000_01b3)
        })
    }

    fn written<T: FileTable>(len: u32, salt: u32) -> Vec<u8> {
        let bytes: Vec<u8> = (0..len)
            .map(|i| (i.wrapping_mul(2_654_435_761).wrapping_add(salt) >> 11) as u8)
            .collect();
        T::read(&bytes).write()
    }

    #[test]
    fn the_best_times_files_tables_are_unchanged() {
        let [
            scores,
            stage_times,
            level_scores,
            course_times,
            alt_times,
            trial_times,
        ] = [1, 2, 3, 4, 7, 8].map(|n| sizes(36)[n]);
        let got = [
            written::<BestScores>(scores, 1),
            written::<StageTimes>(stage_times, 2),
            written::<LevelScores>(level_scores, 3),
            written::<StageTimes>(course_times, 4),
            written::<StageTimes>(alt_times, 5),
            written::<StageTimes>(trial_times, 6),
        ]
        .map(|b| (b.len(), checksum(&b)));
        assert_eq!(
            got,
            [
                (3000, 9_221_644_140_235_737_666),
                (56160, 8_646_818_073_954_991_328),
                (5600, 16_956_960_363_726_488_463),
                (56160, 15_306_710_384_873_672_808),
                (56160, 3_208_713_596_129_006_195),
                (56160, 2_538_053_369_155_243_878),
            ]
        );
    }
}

/// `var\hst.dat` as a whole: a file of the head and the six tables read into typed tables and
/// written out again is the same bytes, for small and large championships.
#[cfg(test)]
mod whole_file {
    use super::*;
    use crate::saves::entries::{LevelScores, StageTimes};
    use crate::saves::kept;
    use crate::saves::scores::BestScores;

    fn varied(len: u32, salt: u32) -> Vec<u8> {
        (0..len)
            .map(|i| (i.wrapping_mul(2_654_435_761).wrapping_add(salt) >> 11) as u8)
            .collect()
    }

    fn file(stages: u32) -> Vec<u8> {
        let mut file = kept::head(kept::BEST_TIMES, 0x0102_0304).to_vec();
        for (salt, n) in [1, 2, 3, 4, 7, 8].into_iter().enumerate() {
            file.extend(varied(sizes(stages)[n], salt as u32 + 1));
        }
        file
    }

    /// The tables read out of `file` and written back, with the head in front.
    fn rewritten(file: &[u8], stages: u32) -> Vec<u8> {
        assert!(kept::head_ok(file, kept::BEST_TIMES));
        let size = |n: usize| sizes(stages)[n] as usize;
        let mut at = kept::HEAD_LEN;
        let mut next = |n: usize| {
            let part = &file[at..at + size(n)];
            at += size(n);
            part
        };
        let mut out = file[..kept::HEAD_LEN].to_vec();
        out.extend(BestScores::read(next(1)).write());
        out.extend(StageTimes::read(next(2)).write());
        out.extend(LevelScores::read(next(3)).write());
        out.extend(StageTimes::read(next(4)).write());
        out.extend(StageTimes::read(next(7)).write());
        out.extend(StageTimes::read(next(8)).write());
        out
    }

    #[test]
    fn a_best_times_file_is_read_and_written_with_the_same_bytes() {
        for stages in [1, 36, 120] {
            let file = file(stages);
            let again = rewritten(&file, stages);
            assert_eq!(again.len(), file.len(), "{stages} stages");
            assert!(again == file, "{stages} stages");
        }
    }
}
