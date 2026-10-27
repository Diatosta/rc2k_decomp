//! The best lists of ten and the records' best holders (converted 2026-10-21 from the
//! arcade's `menu::keep_arcade_stage_best` 0x488213, `check_arcade_level_score` 0x488250,
//! `check_arcade_stage_time` 0x488280, `enter_arcade_stage_time` 0x489F76,
//! `enter_arcade_level_score` 0x48A03F, `clear_arcade_score_marks` 0x48AD08,
//! `clear_arcade_time_marks` 0x48AD30, `take_arcade_stage_best` 0x48ADF8; the records'
//! `menu::clear_record_entries` 0x47202F, `beat_target_time` 0x4A3C23,
//! `enter_record_entry` 0x4A3C7F, `rank_record_entry` 0x4A3D75, `keep_record_best`
//! 0x4A3E8B; the time trial's `menu::time_trial_qualifies` 0x48C7CE,
//! `clear_time_trial_marks` 0x48C85C, `enter_time_trial_best` 0x48C8B6,
//! `rank_time_trial_best` 0x48CC51, `keep_stage_best` 0x48E02D; and the qualifying test of
//! `menu::best_class_scores` 0x4A8608 and `best_overall_scores` 0x4A91BF; the records' pass
//! `menu::record_entries_pass` 0x4A3BE0: `record_entries`).
//!
//! The lists themselves (entering, ranking, clearing the marks) work on the typed entries,
//! `saves::entries`; this holds the tests of a new figure, the new entry's parts, and the
//! records' best holders.

use super::standings::{Entries, NAME_LEN};

/// The 0x40 bytes a best time keeps (its checkpoints; the player's +0x20C).
pub const LINE_LEN: usize = 0x40;

/// Whether a time makes a list whose tenth is `tenth`: at most it (signed).
pub fn makes_time(time: u32, tenth: u32) -> bool {
    time as i32 <= tenth as i32
}

/// Whether a score makes a list whose tenth is `tenth`: at least it (signed).
pub fn makes_score(score: u32, tenth: u32) -> bool {
    score as i32 >= tenth as i32
}

/// The seal a best time keeps: +0x50 the cheats, +0x54 / +0x58 the sealed time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Seal {
    pub cheats: u32,
    pub sealed: [u32; 2],
}

/// A new entry.
pub struct NewBest {
    /// The time or score (+0x14).
    pub key: u32,
    /// The player (+4): 1, or the player's index + 1 for the records.
    pub player: u32,
    /// The player's nationality (+4 of its record; +8 here), name pointer (+0xC), car
    /// (its +8; +0x10 here), group (its +0x90; +0x1C here).
    pub nationality: u32,
    pub name_at: u32,
    pub car: u32,
    pub group: u32,
    /// The name's first bytes (+0x20).
    pub name: [u8; NAME_LEN],
    /// The records' taking part (+0x4C, the player's +0xD0).
    pub entered: Option<u32>,
    /// The best times' seal.
    pub seal: Option<Seal>,
    /// The best times' line (+0x5C).
    pub line: Option<[u8; LINE_LEN]>,
}

/// Where a record keeps its best holder: the figure, nationality, car and the name's 0x20
/// bytes.
pub struct HolderAt {
    pub figure: usize,
    pub nationality: usize,
    pub car: usize,
    pub name: usize,
}

/// The championship's and arcade's records (0xA4 each).
pub const RECORD_HOLDER: HolderAt = HolderAt {
    figure: 0x30,
    nationality: 0x74,
    car: 0x78,
    name: 0x54,
};

/// The time trial's stage records.
pub const STAGE_HOLDER: HolderAt = HolderAt {
    figure: 0x2C,
    nationality: 0x9C,
    car: 0xA0,
    name: 0x7C,
};

/// A best holder: the player's figure (+0x94), nationality (+4), car (+8), name.
pub struct Holder {
    pub figure: u32,
    pub nationality: u32,
    pub car: u32,
    pub name: [u8; 0x20],
}

/// Whether a figure takes a record whose own is `kept`: above it (signed).
pub fn takes_record(figure: u32, kept: u32) -> bool {
    (kept as i32) < figure as i32
}

/// The record (entry `n`) taken by `holder`.
pub fn take_record(record: &mut Entries, n: usize, at: &HolderAt, holder: &Holder) {
    record.store(
        n,
        &[
            (at.figure, holder.figure),
            (at.nationality, holder.nationality),
            (at.car, holder.car),
        ],
    );
    record.set_bytes(n, at.name, &holder.name);
}

/// The record entries' pass (`menu::record_entries_pass`) on the game.
pub trait RecordEntries {
    type Error;
    /// `menu::clear_record_entries`.
    fn clear(&mut self) -> Result<(), Self::Error>;
    /// `dword_755920`: one was entered.
    fn set_entered(&mut self, on: u32);
    /// `dword_75598C`: the current player.
    fn set_current(&mut self, n: u32);
    /// `menu::beat_target_time` for player n (ecx): `dword_755918` 1 after it.
    fn beats(&mut self, n: u32) -> Result<bool, Self::Error>;
    /// `menu::enter_record_entry` for player n.
    fn enter(&mut self, n: u32) -> Result<(), Self::Error>;
}

/// The current event's entries cleared, then each of the `players` counted players that
/// beat the target time entered, `dword_755920` 1 once one is (0 first).
pub fn record_entries<R: RecordEntries>(players: u32, e: &mut R) -> Result<(), R::Error> {
    e.clear()?;
    e.set_entered(0);
    for n in 0..players {
        e.set_current(n);
        if e.beats(n)? {
            e.set_entered(1);
            e.enter(n)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list(bytes: &mut [u8], size: usize) -> Entries<'_> {
        Entries { bytes, size }
    }

    #[test]
    fn tests() {
        assert!(makes_time(5, 5) && !makes_time(6, 5) && makes_time(u32::MAX, 0));
        assert!(makes_score(5, 5) && !makes_score(4, 5) && !makes_score(u32::MAX, 0));
        assert!(takes_record(6, 5) && !takes_record(5, 5) && takes_record(0, u32::MAX));
    }

    #[test]
    fn records() {
        let mut bytes = vec![0u8; 0xA4];
        let mut r = list(&mut bytes, 0xA4);
        let h = Holder {
            figure: 8,
            nationality: 1,
            car: 2,
            name: [b'h'; 0x20],
        };
        take_record(&mut r, 0, &STAGE_HOLDER, &h);
        assert_eq!((r.get(0, 0x2C), r.get(0, 0xA0)), (8, 2));
        assert_eq!(r.bytes_at(0, 0x7C, 0x20), &[b'h'; 0x20]);
    }

    #[test]
    fn record_entries_entered() {
        struct Log(Vec<String>);
        impl RecordEntries for Log {
            type Error = ();
            fn clear(&mut self) -> Result<(), ()> {
                self.0.push("clear".into());
                Ok(())
            }
            fn set_entered(&mut self, on: u32) {
                self.0.push(format!("entered {on}"));
            }
            fn set_current(&mut self, n: u32) {
                self.0.push(format!("current {n}"));
            }
            fn beats(&mut self, n: u32) -> Result<bool, ()> {
                Ok(n == 1)
            }
            fn enter(&mut self, n: u32) -> Result<(), ()> {
                self.0.push(format!("enter {n}"));
                Ok(())
            }
        }
        let mut log = Log(Vec::new());
        record_entries(3, &mut log).unwrap();
        assert_eq!(
            log.0,
            [
                "clear",
                "entered 0",
                "current 0",
                "current 1",
                "entered 1",
                "enter 1",
                "current 2"
            ]
        );
    }
}
