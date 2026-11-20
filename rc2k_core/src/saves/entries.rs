//! The best lists' entry: a best time is 0x9C bytes, its first 0x50 (`Entry`) laid out like an
//! arcade level's best score. Lists are of ten, best first; a new entry goes over the tenth
//! and moves up. The levels' best scores (0x15E0 bytes in memory and in `var\hst.dat`) are
//! seven groups of ten `Entry`s (arcade levels 0 .. 5 and the "class times" page).

use crate::menu::bests::NewBest;

pub const ENTRY_LEN: usize = 0x50;
pub const PER_LIST: usize = 10;
/// The name's room in bytes.
pub const NAME_ROOM: usize = 0x20;
/// What a move takes of a name (and what an entered name gets): its room less the last byte.
pub(crate) const NAME_MOVED: usize = 0x1F;

/// The head of a best entry (0x50 bytes).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Entry {
    /// +0: -1 ends a list.
    pub first: u32,
    /// +4: the player (1 .. 4); 0 for a made-up entry.
    pub player: u32,
    /// +8: the nationality.
    pub nation: u32,
    /// +0xC: the name's string id (made up), or the player's name pointer (entered).
    pub name_id: u32,
    /// +0x10: the car.
    pub car: u32,
    /// +0x14: what ranks it: the time or the score.
    pub key: u32,
    /// +0x18: a rally's table keeps its time here.
    pub rally_time: u32,
    /// +0x1C: the group.
    pub group: u32,
    /// +0x20: the name; a made-up entry's has no NUL, an entered one the player's first 0x1F
    /// bytes.
    pub name: [u8; NAME_ROOM],
    /// +0x40: 1 for the entry just made.
    pub mark: u32,
    /// +0x44.
    pub figure: u32,
    /// +0x48: the name's tail (`name_text` reads on into it).
    pub unread_48: u32,
    /// +0x4C: the records' taking part.
    pub entered: u32,
}

fn dword(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(b[at..at + 4].try_into().expect("four bytes"))
}

fn put(b: &mut [u8], at: usize, v: u32) {
    b[at..at + 4].copy_from_slice(&v.to_le_bytes());
}

/// A field of a best entry, for the lists of fields an exchange moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    /// +0 (table A's sorts move it).
    First,
    Player,
    Nation,
    NameId,
    Car,
    Key,
    RallyTime,
    Group,
    Mark,
    Entered,
    /// A best time's tail: `cheats`, `sealed[0]`, `sealed[1]`, the checkpoints.
    Cheats,
    Sealed0,
    Sealed1,
    Checkpoints,
}

impl Entry {
    /// From its 0x50 bytes.
    pub fn read(b: &[u8]) -> Self {
        let b = &b[..ENTRY_LEN];
        Self {
            first: dword(b, 0),
            player: dword(b, 4),
            nation: dword(b, 8),
            name_id: dword(b, 0xC),
            car: dword(b, 0x10),
            key: dword(b, 0x14),
            rally_time: dword(b, 0x18),
            group: dword(b, 0x1C),
            name: b[0x20..0x40].try_into().expect("the name's room"),
            mark: dword(b, 0x40),
            figure: dword(b, 0x44),
            unread_48: dword(b, 0x48),
            entered: dword(b, 0x4C),
        }
    }

    /// Into its 0x50 bytes.
    pub fn write(&self, b: &mut [u8]) {
        let b = &mut b[..ENTRY_LEN];
        put(b, 0, self.first);
        put(b, 4, self.player);
        put(b, 8, self.nation);
        put(b, 0xC, self.name_id);
        put(b, 0x10, self.car);
        put(b, 0x14, self.key);
        put(b, 0x18, self.rally_time);
        put(b, 0x1C, self.group);
        b[0x20..0x40].copy_from_slice(&self.name);
        put(b, 0x40, self.mark);
        put(b, 0x44, self.figure);
        put(b, 0x48, self.unread_48);
        put(b, 0x4C, self.entered);
    }

    /// Head field `f`'s value (a sort's key).
    pub fn value(&self, f: Field) -> u32 {
        match f {
            Field::First => self.first,
            Field::Player => self.player,
            Field::Nation => self.nation,
            Field::NameId => self.name_id,
            Field::Car => self.car,
            Field::Key => self.key,
            Field::RallyTime => self.rally_time,
            Field::Group => self.group,
            Field::Mark => self.mark,
            Field::Entered => self.entered,
            Field::Cheats | Field::Sealed0 | Field::Sealed1 | Field::Checkpoints => {
                unreachable!("a best time's field asked of a head")
            }
        }
    }

    /// The name's text from +0x20 to the first NUL, which may lie past the name's room. None:
    /// no NUL before the head's end.
    pub fn name_text(&self) -> Option<Vec<u8>> {
        let mut bytes = self.name.to_vec();
        for v in [self.mark, self.figure, self.unread_48, self.entered] {
            bytes.extend(v.to_le_bytes());
        }
        let end = bytes.iter().position(|&c| c == 0)?;
        bytes.truncate(end);
        Some(bytes)
    }

    /// The head of a new entry (`menu::bests::enter_tenth`); a best time's seal and line are
    /// its tail's.
    pub fn enter(&mut self, best: &NewBest) {
        self.key = best.key;
        self.group = best.group;
        self.nation = best.nationality;
        self.name_id = best.name_at;
        self.car = best.car;
        if let Some(entered) = best.entered {
            self.entered = entered;
        }
        self.player = best.player;
        self.mark = 1;
        self.name[..NAME_MOVED].copy_from_slice(&best.name);
    }
}

/// What the lists' code needs of an entry: its head, and its fields exchanged.
pub trait Ranked: Copy {
    fn head(&self) -> &Entry;
    fn head_mut(&mut self) -> &mut Entry;
    /// `f` exchanged between `a` and `b`.
    fn exchange_field(a: &mut Self, b: &mut Self, f: Field);
}

impl Ranked for Entry {
    fn head(&self) -> &Entry {
        self
    }

    fn head_mut(&mut self) -> &mut Entry {
        self
    }

    fn exchange_field(a: &mut Self, b: &mut Self, f: Field) {
        let (x, y) = match f {
            Field::First => (&mut a.first, &mut b.first),
            Field::Player => (&mut a.player, &mut b.player),
            Field::Nation => (&mut a.nation, &mut b.nation),
            Field::NameId => (&mut a.name_id, &mut b.name_id),
            Field::Car => (&mut a.car, &mut b.car),
            Field::Key => (&mut a.key, &mut b.key),
            Field::RallyTime => (&mut a.rally_time, &mut b.rally_time),
            Field::Group => (&mut a.group, &mut b.group),
            Field::Mark => (&mut a.mark, &mut b.mark),
            Field::Entered => (&mut a.entered, &mut b.entered),
            Field::Cheats | Field::Sealed0 | Field::Sealed1 | Field::Checkpoints => {
                unreachable!("a best time's field in a list of heads")
            }
        };
        std::mem::swap(x, y);
    }
}

/// A list of ten.
pub type List = [Entry; PER_LIST];

/// Exchanges entries `a` and `b`'s `fields` and names (the name's last byte stays); the name
/// goes through `kept`, which ends with `a`'s.
pub fn exchange<T: Ranked>(
    list: &mut [T],
    a: usize,
    b: usize,
    fields: &[Field],
    kept: &mut [u8; NAME_MOVED],
) {
    let (mut x, mut y) = (list[a], list[b]);
    for &f in fields {
        T::exchange_field(&mut x, &mut y, f);
    }
    kept.copy_from_slice(&x.head().name[..NAME_MOVED]);
    let from_y: [u8; NAME_MOVED] = y.head().name[..NAME_MOVED].try_into().expect("a name");
    x.head_mut().name[..NAME_MOVED].copy_from_slice(&from_y);
    y.head_mut().name[..NAME_MOVED].copy_from_slice(kept);
    (list[a], list[b]) = (x, y);
}

/// Moves the tenth up: nine steps from the bottom, each pair exchanged when
/// `exchange_if(lower's key, upper's key)`.
pub(crate) fn bubble_up<T: Ranked>(
    list: &mut [T; PER_LIST],
    fields: &[Field],
    exchange_if: fn(u32, u32) -> bool,
    kept: &mut [u8; NAME_MOVED],
) {
    for lower in (1..PER_LIST).rev() {
        let upper = lower - 1;
        if exchange_if(list[lower].head().key, list[upper].head().key) {
            exchange(list, upper, lower, fields, kept);
        }
    }
}

/// The fields an arcade level's score moves (the first eight of the arcade's best times').
pub(crate) const SCORE_FIELDS: [Field; 8] = [
    Field::NameId,
    Field::Key,
    Field::Player,
    Field::Nation,
    Field::Group,
    Field::Car,
    Field::RallyTime,
    Field::Mark,
];

/// Sorts an arcade level's ten scores (a lower score at least the upper's, signed, moves up).
pub fn sort_level_scores(list: &mut List, kept: &mut [u8; NAME_MOVED]) {
    bubble_up(list, &SCORE_FIELDS, |l, u| l as i32 >= u as i32, kept);
}

/// Marks no entry as just made.
pub fn clear_marks<T: Ranked>(list: &mut [T; PER_LIST]) {
    for e in list {
        e.head_mut().mark = 0;
    }
}

pub const LEVELS: usize = 7;
/// The levels' table's bytes in `var\hst.dat`.
pub(crate) const LEVELS_LEN: usize = LEVELS * PER_LIST * ENTRY_LEN;

/// The levels' best scores.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LevelScores {
    pub levels: [List; LEVELS],
}

impl crate::saves::tables::FileTable for LevelScores {
    fn zeroed(bytes: usize) -> Option<Self> {
        (bytes == LEVELS_LEN).then(Self::default)
    }

    fn read(b: &[u8]) -> Self {
        let mut t = Self::default();
        for (n, e) in t.levels.as_flattened_mut().iter_mut().enumerate() {
            *e = Entry::read(&b[n * ENTRY_LEN..]);
        }
        t
    }

    fn write(&self) -> Vec<u8> {
        let mut b = vec![0; LEVELS_LEN];
        for (n, e) in self.levels.as_flattened().iter().enumerate() {
            e.write(&mut b[n * ENTRY_LEN..]);
        }
        b
    }
}

/// The made-up table (`saves::default_class_times`): `entries` (seventy, level by level) over
/// `t`: each its score, group, nationality, car, name id and name.
pub fn fill_made_up(t: &mut LevelScores, entries: &[(u32, crate::saves::defaults::Driver)]) {
    for (e, (key, d)) in t.levels.as_flattened_mut().iter_mut().zip(entries) {
        e.key = *key;
        e.group = d.fields[0];
        e.nation = d.fields[1];
        e.car = d.fields[2];
        e.name_id = d.name_id;
        e.name[..d.name.len()].copy_from_slice(&d.name);
    }
}

/// A best time (156 bytes): the head, then the seal and the checkpoints.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BestTime {
    pub head: Entry,
    /// +0x50: the cheats on when it was driven (a made-up time has 0xB).
    pub cheats: u32,
    /// +0x54, +0x58: the sealed time.
    pub sealed: [u32; 2],
    /// +0x5C: the checkpoint times (the player's line).
    pub checkpoints: [u32; CHECKPOINTS],
}

pub const TIME_LEN: usize = 0x9C;
pub const CHECKPOINTS: usize = 16;

impl BestTime {
    /// From its 0x9C bytes.
    pub fn read(b: &[u8]) -> Self {
        let b = &b[..TIME_LEN];
        Self {
            head: Entry::read(b),
            cheats: dword(b, 0x50),
            sealed: [dword(b, 0x54), dword(b, 0x58)],
            checkpoints: std::array::from_fn(|n| dword(b, 0x5C + 4 * n)),
        }
    }

    /// Into its 0x9C bytes.
    pub fn write(&self, b: &mut [u8]) {
        let b = &mut b[..TIME_LEN];
        self.head.write(b);
        put(b, 0x50, self.cheats);
        put(b, 0x54, self.sealed[0]);
        put(b, 0x58, self.sealed[1]);
        for (n, c) in self.checkpoints.iter().enumerate() {
            put(b, 0x5C + 4 * n, *c);
        }
    }

    /// The name's text from +0x20 to the first NUL. None: no NUL before the entry's end.
    pub fn name_text(&self) -> Option<Vec<u8>> {
        let mut b = [0; TIME_LEN];
        self.write(&mut b);
        let text = &b[0x20..];
        Some(text[..text.iter().position(|&c| c == 0)?].to_vec())
    }

    /// The checkpoints as the 64 bytes they are (the player's line).
    pub fn line(&self) -> [u8; crate::menu::bests::LINE_LEN] {
        let mut b = [0; crate::menu::bests::LINE_LEN];
        for (n, c) in self.checkpoints.iter().enumerate() {
            b[4 * n..4 * n + 4].copy_from_slice(&c.to_le_bytes());
        }
        b
    }

    /// A new best time (`menu::bests::enter_tenth`): the head's part, and the seal and the line
    /// when it has them.
    pub fn enter(&mut self, best: &NewBest) {
        self.head.enter(best);
        if let Some(seal) = best.seal {
            self.cheats = seal.cheats;
            self.sealed = seal.sealed;
        }
        if let Some(line) = best.line {
            self.checkpoints = std::array::from_fn(|n| dword(&line, 4 * n));
        }
    }

    /// A made-up time (`saves::defaults`): the key or, with `rally`, the rally time; the driver's
    /// group, nationality, car and name; cheats 0xB; the guessed checkpoints when it has them.
    pub fn fill_made_up(&mut self, e: &crate::saves::defaults::TimeEntry, rally: bool) {
        let h = &mut self.head;
        if rally {
            h.rally_time = e.time;
        } else {
            h.key = e.time;
        }
        h.group = e.driver.fields[0];
        h.nation = e.driver.fields[1];
        self.cheats = 0xB;
        if let Some(c) = e.checkpoints {
            self.checkpoints = c;
        }
        let h = &mut self.head;
        h.car = e.driver.fields[2];
        h.name_id = e.driver.name_id;
        h.name[..e.driver.name.len()].copy_from_slice(&e.driver.name);
    }
}

impl Ranked for BestTime {
    fn head(&self) -> &Entry {
        &self.head
    }

    fn head_mut(&mut self) -> &mut Entry {
        &mut self.head
    }

    fn exchange_field(a: &mut Self, b: &mut Self, f: Field) {
        match f {
            Field::Cheats => std::mem::swap(&mut a.cheats, &mut b.cheats),
            Field::Sealed0 => std::mem::swap(&mut a.sealed[0], &mut b.sealed[0]),
            Field::Sealed1 => std::mem::swap(&mut a.sealed[1], &mut b.sealed[1]),
            Field::Checkpoints => std::mem::swap(&mut a.checkpoints, &mut b.checkpoints),
            head => Entry::exchange_field(&mut a.head, &mut b.head, head),
        }
    }
}

/// A list of ten best times.
pub type Times = [BestTime; PER_LIST];

/// The fields a record's entry moves (`menu::rank_record_entry`).
pub(crate) const RECORD_FIELDS: [Field; 12] = [
    Field::NameId,
    Field::Key,
    Field::Player,
    Field::Nation,
    Field::Group,
    Field::Car,
    Field::RallyTime,
    Field::Mark,
    Field::Entered,
    Field::Cheats,
    Field::Sealed0,
    Field::Sealed1,
];

/// The fields the arcade's and the time trial's best times move.
pub(crate) const TIME_FIELDS: [Field; 12] = [
    Field::NameId,
    Field::Key,
    Field::Player,
    Field::Nation,
    Field::Group,
    Field::Car,
    Field::RallyTime,
    Field::Mark,
    Field::Cheats,
    Field::Sealed0,
    Field::Sealed1,
    Field::Checkpoints,
];

/// A record's ten: the tenth moved up (a lower time at most the upper's, signed).
pub fn rank_record(list: &mut Times, kept: &mut [u8; NAME_MOVED]) {
    bubble_up(list, &RECORD_FIELDS, |l, u| l as i32 <= u as i32, kept);
}

/// A stage's ten best times: the tenth moved up (a lower time below the upper's, unsigned).
pub fn rank_time_trial(list: &mut Times, kept: &mut [u8; NAME_MOVED]) {
    bubble_up(list, &TIME_FIELDS, |l, u| l < u, kept);
}

/// The arcade course's ten best times sorted (a lower time at most the upper's, signed).
pub fn sort_arcade_times(list: &mut Times, kept: &mut [u8; NAME_MOVED]) {
    bubble_up(list, &TIME_FIELDS, |l, u| l as i32 <= u as i32, kept);
}

/// The best times of the stages: ten for each stage.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StageTimes {
    pub stages: Vec<Times>,
}

/// A stage's ten entries in the file, in bytes.
pub(crate) const STAGE_LEN: usize = PER_LIST * TIME_LEN;

impl StageTimes {
    /// The zero bytes the block holds past the table (its pages less its header): what a read
    /// just past the table finds.
    fn slack(&self) -> usize {
        let len = self.stages.len() * STAGE_LEN;
        let header = crate::memory::HEADER_LEN as usize;
        (len + header).next_multiple_of(0x1000) - header - len
    }

    /// The first dword of entry `n` counted from the first stage's, for the lists' look past
    /// their ten: zeros past the stages. None: past the block.
    pub fn first_at(&self, n: usize) -> Option<u32> {
        match self.stages.as_flattened().get(n) {
            Some(e) => Some(e.head.first),
            None => {
                let past = (n + 1) * TIME_LEN - self.stages.len() * STAGE_LEN;
                (past <= self.slack()).then_some(0)
            }
        }
    }
}

impl crate::saves::tables::FileTable for StageTimes {
    fn zeroed(bytes: usize) -> Option<Self> {
        bytes.is_multiple_of(STAGE_LEN).then(|| Self {
            stages: vec![Times::default(); bytes / STAGE_LEN],
        })
    }

    fn read(b: &[u8]) -> Self {
        Self {
            stages: b
                .as_chunks::<STAGE_LEN>()
                .0
                .iter()
                .map(|s| std::array::from_fn(|n| BestTime::read(&s[n * TIME_LEN..])))
                .collect(),
        }
    }

    fn write(&self) -> Vec<u8> {
        let mut b = vec![0; self.stages.len() * STAGE_LEN];
        for (n, e) in self.stages.as_flattened().iter().enumerate() {
            e.write(&mut b[n * TIME_LEN..]);
        }
        b
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::saves::tables::FileTable;

    #[test]
    fn bytes_round_trip_at_the_originals_offsets() {
        let b: Vec<u8> = (0..LEVELS_LEN).map(|n| (n * 13) as u8).collect();
        let t = LevelScores::read(&b);
        let at = 23 * ENTRY_LEN;
        let e = t.levels[2][3];
        assert_eq!(e.key, dword(&b, at + 0x14));
        assert_eq!(e.group, dword(&b, at + 0x1C));
        assert_eq!(e.mark, dword(&b, at + 0x40));
        assert_eq!(e.name[..], b[at + 0x20..at + 0x40]);
        assert_eq!(t.write(), b);
    }

    #[test]
    fn entering_and_ranking_times() {
        let mut list: Times = std::array::from_fn(|n| {
            let mut e = BestTime::default();
            e.head.key = 100 * (n as u32 + 1);
            e.head.mark = 7;
            e
        });
        clear_marks(&mut list);
        assert!(list.iter().all(|e| e.head.mark == 0));
        let best = NewBest {
            key: 250,
            player: 1,
            nationality: 3,
            name_at: 0x4000,
            car: 4,
            group: 2,
            name: [b'z'; NAME_MOVED],
            entered: None,
            seal: Some(crate::menu::bests::Seal {
                cheats: 9,
                sealed: [10, 11],
            }),
            line: Some([0x5A; 0x40]),
        };
        list[9].enter(&best);
        assert_eq!(
            (list[9].head.name[0], list[9].sealed[0], list[9].head.mark),
            (b'z', 10, 1)
        );
        let mut kept = [0; NAME_MOVED];
        rank_time_trial(&mut list, &mut kept);
        assert_eq!(
            (list[2].head.key, list[2].sealed[1], list[2].head.mark),
            (250, 11, 1)
        );
        assert_eq!(list[0].line(), [0; 0x40]);
        assert_eq!(list[2].checkpoints[0], 0x5A5A_5A5A);
        // A record's ranking leaves the checkpoints; equal times move up.
        list[9].head.key = 300;
        rank_record(&mut list, &mut kept);
        assert_eq!((list[3].head.key, list[4].head.key), (300, 300));
        assert_eq!(list[3].checkpoints, [0; CHECKPOINTS]);
    }

    #[test]
    fn made_up_times_match_the_byte_version() {
        use crate::saves::defaults::{Driver, TimeEntry};
        let e = TimeEntry {
            time: 0x1234,
            driver: Driver {
                fields: [1, 2, 3],
                name_id: 4,
                name: b"Someone".to_vec(),
            },
            checkpoints: Some(std::array::from_fn(|n| n as u32 * 3)),
        };
        for (time_at, rally) in [(0x14, false), (0x18, true)] {
            let mut bytes: Vec<u8> = (0..TIME_LEN).map(|n| n as u8).collect();
            let mut typed = BestTime::read(&bytes);
            e.write(&mut bytes, time_at);
            typed.fill_made_up(&e, rally);
            let mut out = vec![0; TIME_LEN];
            typed.write(&mut out);
            assert_eq!(out, bytes);
        }
    }

    #[test]
    fn stage_tables_read_zeros_just_past_their_end() {
        use crate::saves::tables::FileTable;
        let mut t = StageTimes::zeroed(2 * STAGE_LEN).unwrap();
        t.stages[1][9].head.first = 7;
        assert_eq!(t.first_at(19), Some(7));
        // 0xC30 bytes: the block's 0x1000 less its 0x40 header leave 0x390 more.
        assert_eq!(t.first_at(20), Some(0));
        assert_eq!(t.first_at(20 + 0x390 / TIME_LEN - 1), Some(0));
        assert_eq!(t.first_at(20 + 0x390 / TIME_LEN), None);
        assert_eq!(StageTimes::read(&t.write()), t);
        assert!(StageTimes::zeroed(5).is_none());
    }
}
