//! Made-up best times and scores, for a game without `var\hst.dat`.
//!
//! Every entry gets a random driver. A time entry may also get checkpoint times guessed from
//! its time, while the game's switch for them is on. Everything is worked out before anything
//! is written: a refusal leaves the tables as they were.

use crate::Invalid;

/// A time entry's length.
pub const ENTRY_LEN: usize = 0x9C;
/// The stages a stage table has room for.
pub(crate) const STAGE_ROOM: u32 = 36;
/// A rally's table: twenty time entries.
pub(crate) const RALLY_ENTRIES: usize = 20;
/// The number of rallies with a table.
pub const RALLIES: usize = 6;
pub const CHECKPOINTS: usize = 16;
/// The last stage with a checkpoint count; past it the guesses are all 0.
const LAST_COUNTED_STAGE: u32 = 0x24;
/// Added to a record's time (24 seconds, in milliseconds).
const SLACK: u32 = 0x5DC0;
/// How many drivers a plain random driver is drawn from, and a score's first draw.
const DRIVER_DRAW: u32 = 0xAA;
const SCORE_DRAW: u32 = 0xB3;

/// The texts the guessing writes to the log.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Text {
    /// A line of dashes, before the guess.
    Dashes,
    /// "guesscheckpointtimes:start"
    Start,
    /// "passing:(time:...)"
    Passing,
    /// "(stage:...)"
    Stage,
    /// "testcheckdata"
    CheckData,
    /// "guesscheckpointtimes:done"
    Done,
    /// A line of dashes, after.
    DashesAfter,
}

/// A piece written to the game's log, with or without a new line after it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Piece {
    Text(Text, bool),
    Number(u32, bool),
}

/// What an entry takes from a driver's record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Driver {
    /// Fields +8, +0x14, +0x18 of the driver's record.
    pub fields: [u32; 3],
    pub name_id: u32,
    /// Without a NUL; at most 0x20 bytes (the adapter refuses longer).
    pub name: Vec<u8>,
}

/// Where the functions get what is not theirs.
pub trait Source {
    /// The game's random number under `limit`.
    fn below(&mut self, limit: u32) -> Result<u32, Invalid>;
    /// Driver `number`'s record (refused past the 180).
    fn driver(&mut self, number: u32) -> Result<Driver, Invalid>;
    /// How many checkpoints `stage` (at most 36) has.
    fn checkpoint_count(&mut self, stage: u32) -> u32;
    fn log(&mut self, piece: Piece);
}

/// A championship record (164 bytes) as these functions read it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Record {
    /// 0 a stage, 3 or 4 a rally's end.
    pub tag: u32,
    /// The stage's track.
    pub track: u32,
    /// The stage's time (or length).
    pub length: u32,
    /// The record's time.
    pub time: u32,
    /// Which ten entries of a stage table.
    pub picked: u32,
}

fn put(bytes: &mut [u8], at: usize, value: u32) {
    bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
}

/// Checkpoint n of `count` at `time` * (n + 1) / count, the rest 0.
pub(crate) fn guess_checkpoints(time: u32, count: u32) -> [u32; CHECKPOINTS] {
    let mut guessed = [0; CHECKPOINTS];
    for (n, g) in guessed.iter_mut().enumerate().take(count as usize) {
        *g = ((n as u64 + 1) * u64::from(time) / u64::from(count)) as u32;
    }
    guessed
}

/// A time entry: its time, its driver and, with the switch on, its checkpoints.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimeEntry {
    pub time: u32,
    pub driver: Driver,
    pub checkpoints: Option<[u32; CHECKPOINTS]>,
}

impl TimeEntry {
    /// Into its 0x9C bytes; the time goes at `time_at` (+0x14 in a stage table, +0x18 in a
    /// rally's).
    pub fn write(&self, entry: &mut [u8], time_at: usize) {
        put(entry, time_at, self.time);
        put(entry, 0x1C, self.driver.fields[0]);
        put(entry, 8, self.driver.fields[1]);
        put(entry, 0x50, 0xB);
        if let Some(checkpoints) = self.checkpoints {
            for (n, c) in checkpoints.iter().enumerate() {
                put(entry, 0x5C + n * 4, *c);
            }
        }
        put(entry, 0x10, self.driver.fields[2]);
        put(entry, 0xC, self.driver.name_id);
        entry[0x20..0x20 + self.driver.name.len()].copy_from_slice(&self.driver.name);
    }
}

/// A random driver.
fn random_driver(src: &mut dyn Source) -> Result<Driver, Invalid> {
    let number = src.below(DRIVER_DRAW)?;
    src.driver(number)
}

/// A time entry: with `with_checkpoints` the guessed checkpoints (logged), then the driver.
pub fn time_entry(
    src: &mut dyn Source,
    time: u32,
    stage: u32,
    with_checkpoints: bool,
) -> Result<TimeEntry, Invalid> {
    let checkpoints = if with_checkpoints {
        src.log(Piece::Text(Text::Dashes, true));
        src.log(Piece::Text(Text::Start, true));
        src.log(Piece::Text(Text::Passing, false));
        src.log(Piece::Number(time, false));
        src.log(Piece::Text(Text::Stage, false));
        src.log(Piece::Number(stage, true));
        let count = if stage > LAST_COUNTED_STAGE {
            0
        } else {
            match src.checkpoint_count(stage) {
                0 => return Err(Invalid("a stage with no checkpoints")),
                n if n > CHECKPOINTS as u32 => {
                    return Err(Invalid("a stage with more than sixteen checkpoints"));
                }
                n => n,
            }
        };
        let guessed = guess_checkpoints(time, count);
        src.log(Piece::Text(Text::CheckData, false));
        for (n, g) in guessed.iter().enumerate() {
            src.log(Piece::Number(*g, n % 8 == 7));
        }
        src.log(Piece::Text(Text::Done, true));
        src.log(Piece::Text(Text::DashesAfter, true));
        Some(guessed)
    } else {
        None
    };
    Ok(TimeEntry {
        time,
        driver: random_driver(src)?,
        checkpoints,
    })
}

/// `count` entries from `time` on, `step` apart.
fn entries(
    src: &mut dyn Source,
    count: usize,
    time: u32,
    step: u32,
    stage: u32,
    with_checkpoints: bool,
) -> Result<Vec<TimeEntry>, Invalid> {
    (0..count as u32)
        .map(|n| time_entry(src, time.wrapping_add(n * step), stage, with_checkpoints))
        .collect()
}

/// Where the first entry goes (its number) and where an entry's time is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    /// 0: the function's own table; 1 .. 6: the rallies' tables.
    pub table: usize,
    pub first: usize,
    pub time_at: usize,
    pub entries: Vec<TimeEntry>,
}

impl Block {
    /// Into the table's bytes.
    pub fn write(&self, table: &mut [u8]) {
        for (n, e) in self.entries.iter().enumerate() {
            let at = (self.first + n) * ENTRY_LEN;
            e.write(&mut table[at..at + ENTRY_LEN], self.time_at);
        }
    }
}

/// What a function of the time tables leaves: its blocks in order (a later one may cover an
/// earlier one) and the work variables as it ends.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Filled {
    pub blocks: Vec<Block>,
    /// The made-up times' seconds.
    pub seconds: u32,
    /// The sum kept in `DrawWork.button_x`, when the function writes it.
    pub sum: Option<u32>,
    /// The last guess, when one was made.
    pub checkpoints: Option<[u32; CHECKPOINTS]>,
}

impl Filled {
    fn new() -> Self {
        Filled {
            blocks: Vec::new(),
            seconds: 0,
            sum: None,
            checkpoints: None,
        }
    }

    fn push(&mut self, block: Block) {
        if let Some(c) = block.entries.iter().rev().find_map(|e| e.checkpoints) {
            self.checkpoints = Some(c);
        }
        self.blocks.push(block);
    }
}

/// For each table's records 1 .. 6, ten entries at the record's time + 24 s, 0x8B apart, in the
/// ten its +0x3C picks; `stage` is the stage, or None for each record's own pick. `tables` holds
/// the six tables' records 1 .. 6.
pub fn stage_times(
    src: &mut dyn Source,
    tables: &[[Record; 6]],
    stage: Option<u32>,
    with_checkpoints: bool,
) -> Result<Filled, Invalid> {
    let mut filled = Filled::new();
    for record in tables.iter().flatten() {
        let time = record.length.wrapping_add(SLACK);
        filled.seconds = time;
        if record.picked >= STAGE_ROOM {
            return Err(Invalid("a stage past the table"));
        }
        let stage = stage.unwrap_or(record.picked);
        let block = Block {
            table: 0,
            first: record.picked as usize * 10,
            time_at: 0x14,
            entries: entries(src, 10, time, 0x8B, stage, with_checkpoints)?,
        };
        filled.push(block);
    }
    Ok(filled)
}

/// Which of the two championship functions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Championship {
    /// The record's +0x24, the stage for the checkpoints the caller's argument, the rallies'
    /// tables filled.
    Times { stage: u32 },
    /// The record's +0x18, the stage its track; the rallies' sum made and thrown away.
    Checkpoints,
}

/// The championship's stages (`records` up to its end, tag 0): ten entries each from the
/// record's time + 24 s, 0x9F apart; then, adding up each stage's +0x18 + 0xB6, at each rally's
/// end (tag 3 or 4) the sum is used (`Times`: twenty entries of the next rally's table, 0xB2
/// apart, the time at +0x18) and restarted. `rally_tables`: which of the six tables exist.
pub fn championship_times(
    src: &mut dyn Source,
    records: &[Record],
    which: Championship,
    rally_tables: [bool; RALLIES],
    with_checkpoints: bool,
) -> Result<Filled, Invalid> {
    let stages: Vec<&Record> = records.iter().filter(|r| r.tag == 0).collect();
    if stages.is_empty() {
        return Err(Invalid("no stages"));
    }
    let mut filled = Filled::new();
    filled.seconds = stages.len() as u32;
    for (n, record) in stages.iter().enumerate() {
        let (time, stage) = match which {
            Championship::Times { stage } => (record.time, stage),
            Championship::Checkpoints => (record.length, record.track),
        };
        let block = Block {
            table: 0,
            first: n * 10,
            time_at: 0x14,
            entries: entries(
                src,
                10,
                time.wrapping_add(SLACK),
                0x9F,
                stage,
                with_checkpoints,
            )?,
        };
        filled.push(block);
    }
    let mut sum = 0u32;
    let mut rally = 0;
    for record in records {
        match record.tag {
            0 => sum = sum.wrapping_add(record.length.wrapping_add(0xB6)),
            3 | 4 => {
                if let Championship::Times { stage } = which {
                    if rally == RALLIES {
                        return Err(Invalid("more rallies than tables for them"));
                    }
                    if !rally_tables[rally] {
                        return Err(Invalid("a rally without its table"));
                    }
                    let block = Block {
                        table: rally + 1,
                        first: 0,
                        time_at: 0x18,
                        entries: entries(src, RALLY_ENTRIES, sum, 0xB2, stage, with_checkpoints)?,
                    };
                    filled.push(block);
                    rally += 1;
                }
                sum = 0;
            }
            _ => {}
        }
    }
    filled.sum = Some(sum);
    Ok(filled)
}

/// Seven groups of ten, from 100,000 down by 0xAB3, each with a random driver.
pub fn class_times(src: &mut dyn Source) -> Result<Vec<(u32, Driver)>, Invalid> {
    let mut entries = Vec::new();
    for _ in 0..7 {
        for n in 0..10 {
            entries.push((100_000 - n * 0xAB3, random_driver(src)?));
        }
    }
    Ok(entries)
}

/// The drivers of each of the four classes. Fixed: the 180 drivers are 90, 50, 30 and 10.
pub const CLASS_DRIVER_COUNTS: [u32; 4] = [0x5A, 0x32, 0x1E, 0x0A];

/// The first driver of each class in the drivers table.
pub const CLASS_FIRST_DRIVERS: [u32; 4] = [0, 0x5A, 0x8C, 0xAA];

/// Five groups of ten, scores 50 down to 41. Each entry draws a number under 0xB3; in groups
/// 0 .. 3 it is thrown away and the driver is one of the group's `counts[group]` from
/// `firsts[group]` on. Refused (before any draw): a group without drivers.
pub fn best_scores(
    src: &mut dyn Source,
    counts: [u32; 4],
    firsts: [u32; 4],
) -> Result<Vec<(u32, Driver)>, Invalid> {
    if counts.contains(&0) {
        return Err(Invalid("a group without drivers"));
    }
    let mut entries = Vec::new();
    for group in 0..5 {
        for n in 0..10 {
            let mut number = src.below(SCORE_DRAW)?;
            if group != 4 {
                number = src.below(counts[group])?.wrapping_add(firsts[group]);
            }
            entries.push((0x32 - n, src.driver(number)?));
        }
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Draws 0, 1, 2, ..; driver n has fields n, n + 1, n + 2, name "Dn".
    struct Test {
        next: u32,
        log: Vec<Piece>,
        counts: u32,
    }

    impl Source for Test {
        fn below(&mut self, limit: u32) -> Result<u32, Invalid> {
            if limit == 0 {
                return Err(Invalid("0"));
            }
            self.next += 1;
            Ok((self.next - 1) % limit)
        }
        fn driver(&mut self, number: u32) -> Result<Driver, Invalid> {
            if number >= 180 {
                return Err(Invalid("past"));
            }
            Ok(Driver {
                fields: [number, number + 1, number + 2],
                name_id: 0x100 + number,
                name: format!("D{number}").into_bytes(),
            })
        }
        fn checkpoint_count(&mut self, _: u32) -> u32 {
            self.counts
        }
        fn log(&mut self, piece: Piece) {
            self.log.push(piece);
        }
    }

    fn test() -> Test {
        Test {
            next: 0,
            log: Vec::new(),
            counts: 4,
        }
    }

    #[test]
    fn guesses() {
        assert_eq!(guess_checkpoints(1000, 4)[..5], [250, 500, 750, 1000, 0]);
        assert_eq!(guess_checkpoints(u32::MAX, 1)[0], u32::MAX);
        assert_eq!(guess_checkpoints(5, 0), [0; 16]);
    }

    #[test]
    fn a_time_entry() {
        let mut t = test();
        let e = time_entry(&mut t, 1000, 3, true).unwrap();
        assert_eq!(e.checkpoints.unwrap()[..4], [250, 500, 750, 1000]);
        assert_eq!(t.log.len(), 6 + 1 + 16 + 2);
        assert_eq!(t.log[3], Piece::Number(1000, false));
        assert_eq!(t.log[14], Piece::Number(0, true));
        let mut bytes = [0xAA; ENTRY_LEN];
        e.write(&mut bytes, 0x14);
        assert_eq!(bytes[0x14..0x18], 1000u32.to_le_bytes());
        assert_eq!(bytes[0x50], 0xB);
        assert_eq!(&bytes[0x20..0x22], b"D0");
        assert_eq!(bytes[0x22], 0xAA);
        // Past 0x24: no count asked, all 0; a count of 0 or past 16 refused.
        t.counts = 0;
        assert_eq!(
            time_entry(&mut t, 9, 0x25, true).unwrap().checkpoints,
            Some([0; 16])
        );
        assert!(time_entry(&mut t, 9, 0x24, true).is_err());
        t.counts = 17;
        assert!(time_entry(&mut t, 9, 1, true).is_err());
        let n = t.log.len();
        assert_eq!(time_entry(&mut t, 9, 1, false).unwrap().checkpoints, None);
        assert_eq!(t.log.len(), n);
    }

    #[test]
    fn stages() {
        let r = |length, picked| Record {
            length,
            picked,
            ..Record::default()
        };
        let tables = [[r(1, 0), r(2, 1), r(3, 2), r(4, 3), r(5, 4), r(6, 35)]];
        let f = stage_times(&mut test(), &tables, None, false).unwrap();
        assert_eq!(f.blocks.len(), 6);
        assert_eq!(f.seconds, 6 + SLACK);
        assert_eq!(f.blocks[5].first, 350);
        assert_eq!(f.blocks[1].entries[9].time, 2 + SLACK + 9 * 0x8B);
        let bad = [[r(1, 0), r(2, 36), r(3, 2), r(4, 3), r(5, 4), r(6, 5)]];
        assert!(stage_times(&mut test(), &bad, None, false).is_err());
    }

    #[test]
    fn championship() {
        let s = |length, time| Record {
            length,
            time,
            track: 7,
            ..Record::default()
        };
        let end = Record {
            tag: 3,
            ..Record::default()
        };
        let records = [
            s(10, 100),
            s(20, 200),
            end,
            s(30, 300),
            Record { tag: 4, ..end },
            s(1, 1),
        ];
        let f = championship_times(
            &mut test(),
            &records,
            Championship::Times { stage: 2 },
            [true; 6],
            false,
        )
        .unwrap();
        assert_eq!(f.seconds, 4);
        assert_eq!(f.blocks.len(), 6);
        assert_eq!(f.blocks[1].entries[0].time, 200 + SLACK);
        assert_eq!(f.blocks[4].table, 1);
        assert_eq!(f.blocks[4].entries[0].time, 30 + 0xB6 * 2);
        assert_eq!(f.blocks[5].entries[19].time, 30 + 0xB6 + 19 * 0xB2);
        assert_eq!(f.sum, Some(1 + 0xB6));
        let c = championship_times(
            &mut test(),
            &records,
            Championship::Checkpoints,
            [false; 6],
            false,
        )
        .unwrap();
        assert_eq!(c.blocks.len(), 4);
        assert_eq!(c.blocks[0].entries[0].time, 10 + SLACK);
        let mut no_table = [true; 6];
        no_table[1] = false;
        assert!(
            championship_times(
                &mut test(),
                &records,
                Championship::Times { stage: 0 },
                no_table,
                false
            )
            .is_err()
        );
        assert!(
            championship_times(
                &mut test(),
                &[end],
                Championship::Checkpoints,
                [true; 6],
                false
            )
            .is_err()
        );
    }

    #[test]
    fn class_and_scores() {
        let c = class_times(&mut test()).unwrap();
        assert_eq!(c.len(), 70);
        assert_eq!(c[9].0, 100_000 - 9 * 0xAB3);
        let mut table = crate::saves::entries::LevelScores::default();
        crate::saves::entries::fill_made_up(&mut table, &c);
        assert_eq!(table.levels[0][1].key, 100_000 - 0xAB3);
        let s = best_scores(&mut test(), [5, 5, 5, 5], [10, 20, 30, 40]).unwrap();
        assert_eq!(s.len(), 50);
        // Entry 0: draws 0 (thrown), then 1 % 5 + 10.
        assert_eq!(s[0].1.fields[0], 11);
        assert_eq!(s[40].0, 0x32);
        assert!(best_scores(&mut test(), [5, 0, 5, 5], [0; 4]).is_err());
        let mut table = crate::saves::scores::BestScores::default();
        crate::saves::scores::fill_made_up(&mut table, &s);
        assert_eq!(table.groups[0][0].score, 0x32);
    }
}
