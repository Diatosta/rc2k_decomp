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
//! `menu::best_class_scores` 0x4A8608 and `best_overall_scores` 0x4A91BF).
//!
//! A list is ten entries (best times 0x9C each, the arcade's best scores 0x50): +0x14 the
//! time or score, +4 the player, +8 the nationality, +0xC the name's pointer, +0x10 the
//! car, +0x1C the group, +0x20 the name, +0x40 the new entry's mark. A new entry goes in
//! the tenth and moves up.

use super::standings::{ARCADE_TIME_FIELDS, Entries, NAME_LEN, bubble_up};

/// The entry a new one goes in.
pub const TENTH: usize = 9;
/// The time or score.
pub const KEY: usize = 0x14;
/// The new entry's mark (1 for the one just entered).
pub const MARK: usize = 0x40;
/// The names.
pub const NAME: usize = 0x20;
/// The 0x40 bytes a best time keeps at +0x5C (the player's +0x20C).
pub const LINE: usize = 0x5C;
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

/// The new entry stored in the list's tenth, marked (+0x40 1). The name pointer the
/// original stores at +0x20 first is covered by the name.
pub fn enter_tenth(list: &mut Entries, best: &NewBest) {
    list.store(
        TENTH,
        &[
            (KEY, best.key),
            (0x1C, best.group),
            (8, best.nationality),
            (0xC, best.name_at),
            (0x10, best.car),
        ],
    );
    if let Some(entered) = best.entered {
        list.set(TENTH, 0x4C, entered);
    }
    if let Some(seal) = best.seal {
        list.store(
            TENTH,
            &[
                (0x50, seal.cheats),
                (0x54, seal.sealed[0]),
                (0x58, seal.sealed[1]),
            ],
        );
    }
    list.store(TENTH, &[(4, best.player), (MARK, 1)]);
    if let Some(line) = best.line {
        list.set_bytes(TENTH, LINE, &line);
    }
    list.set_name(TENTH, NAME, &best.name);
}

/// Every entry's mark cleared.
pub fn clear_marks(list: &mut Entries) {
    list.store_all(&[(MARK, 0)]);
}

/// The first entry's line (`menu::take_arcade_stage_best`).
pub fn best_line(list: &Entries) -> [u8; LINE_LEN] {
    list.bytes_at(0, LINE, LINE_LEN).try_into().unwrap()
}

/// The record entries' fields an exchange moves (the names too).
pub const RECORD_FIELDS: [usize; 12] = [
    0xC, 0x14, 4, 8, 0x1C, 0x10, 0x18, 0x40, 0x4C, 0x50, 0x54, 0x58,
];

/// A record's ten entries' tenth moved up: a lower time at most the upper's (signed)
/// exchanges.
pub fn rank_record(list: &mut Entries, kept: &mut [u8; NAME_LEN]) {
    bubble_up(list, KEY, &RECORD_FIELDS, NAME, |l, u| l <= u, kept);
}

/// A stage's best times' tenth moved up: a lower time below the upper's (unsigned)
/// exchanges, the fields of the arcade's best times.
pub fn rank_time_trial(list: &mut Entries, kept: &mut [u8; NAME_LEN]) {
    bubble_up(
        list,
        KEY,
        &ARCADE_TIME_FIELDS,
        NAME,
        |l, u| (l as u32) < (u as u32),
        kept,
    );
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
    fn entering_and_ranking() {
        let mut bytes = vec![0u8; 10 * 0x9C];
        let mut t = list(&mut bytes, 0x9C);
        for n in 0..10 {
            t.set(n, KEY, 100 * (n as u32 + 1));
            t.set(n, MARK, 7);
        }
        clear_marks(&mut t);
        assert!((0..10).all(|n| t.get(n, MARK) == 0));
        let best = NewBest {
            key: 250,
            player: 1,
            nationality: 3,
            name_at: 0x4000,
            car: 4,
            group: 2,
            name: [b'z'; NAME_LEN],
            entered: None,
            seal: Some(Seal {
                cheats: 9,
                sealed: [10, 11],
            }),
            line: Some([0x5A; LINE_LEN]),
        };
        enter_tenth(&mut t, &best);
        assert_eq!(
            (t.get(9, 0x20), t.get(9, 0x54), t.get(9, MARK)),
            (0x7A7A_7A7A, 10, 1)
        );
        let mut kept = [0; NAME_LEN];
        rank_time_trial(&mut t, &mut kept);
        assert_eq!(
            (t.get(2, KEY), t.get(2, 0x58), t.get(2, MARK)),
            (250, 11, 1)
        );
        assert_eq!(best_line(&t), [0; LINE_LEN]);
        assert_eq!(t.bytes_at(2, LINE, 2), [0x5A, 0x5A]);
        t.set(9, KEY, 300);
        rank_record(&mut t, &mut kept);
        assert_eq!((t.get(3, KEY), t.get(4, KEY)), (300, 300));
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
}
