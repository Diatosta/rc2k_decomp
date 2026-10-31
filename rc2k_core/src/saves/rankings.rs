//! The championship's working tables (typed 2026-10-27, phase 12): the ranked times
//! (`g_rankedTimes`, standings table A), the championship's history (`g_champHistory`,
//! standings table B) and the summary the race takes its rivals from (`dword_7556F4`). Not
//! in `var\hst.dat`; taken with the best times' tables (`saves::allocate_best_times`, the
//! nine's first, sixth and seventh).
//!
//! Tables A and B have room for `ROOM` entries and a last dword of -1 (the allocation's
//! mark); the walks stop at an entry whose first dword is -1, which `init_standings_*`
//! make of every entry from the 200th down to the first before filling.

use crate::saves::entries::{BestTime, Field, NAME_MOVED, TIME_LEN};

/// The entries tables A and B have room for.
pub const ROOM: usize = 2000;
/// A walk that meets no end entry in this many is refused.
pub const MOST_WALKED: usize = 0x400;

fn dword(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(b[at..at + 4].try_into().expect("four bytes"))
}

fn put(b: &mut [u8], at: usize, v: u32) {
    b[at..at + 4].copy_from_slice(&v.to_le_bytes());
}

/// What the tables' walks need: an entry's first dword (-1 ends a list).
pub trait Walked {
    fn first(&self) -> u32;
}

impl Walked for BestTime {
    fn first(&self) -> u32 {
        self.head.first
    }
}

/// A table of `ROOM` entries and its last dword.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Table<T> {
    pub entries: Vec<T>,
    /// The dword after them (-1 from the allocation).
    pub end: u32,
}

impl<T: Walked + Default + Clone> Table<T> {
    fn new() -> Self {
        Self {
            entries: vec![T::default(); ROOM],
            end: u32::MAX,
        }
    }

    /// Entry `n`'s first dword, the table's last dword after the entries; None past it.
    pub fn first_at(&self, n: usize) -> Option<u32> {
        match self.entries.get(n) {
            Some(e) => Some(e.first()),
            None => (n == self.entries.len()).then_some(self.end),
        }
    }

    /// How many entries the original's `do .. while` walks: the first always, then each
    /// while its first dword is not -1. None: no end within `MOST_WALKED` (refused).
    pub fn walked(&self) -> Option<usize> {
        (1..=MOST_WALKED).find(|&n| self.first_at(n) == Some(u32::MAX))
    }
}

/// Table A: the ranked times, laid out as best times (+0x14 the stage time, +0x18 the
/// rally's, +0x1C the group, +0x44 the position, +0x4C the taking part).
pub type RankedTimes = Table<BestTime>;

/// A championship history entry (table B, 0x78 bytes).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HistoryEntry {
    /// +0: the driver's number; -1 ends the list.
    pub first: u32,
    /// +4: the player (1 .. 4), 0 a computer driver.
    pub player: u32,
    /// +8: the nationality.
    pub nation: u32,
    /// +0xC: the car.
    pub car: u32,
    /// +0x10: the group.
    pub group: u32,
    /// +0x14: the name (0x20 bytes of room).
    pub name: [u8; 0x20],
    /// +0x34: the total.
    pub total: u32,
    /// +0x38: the overall points of the six rounds, then their total (+0x50).
    pub overall: [u32; 7],
    /// +0x54: the stage points of the six rounds, then their total (+0x6C).
    pub stage: [u32; 7],
    /// +0x70: the position.
    pub position: u32,
    /// +0x74: logged as the driver's status.
    pub status: u32,
}

pub const HISTORY_LEN: usize = 0x78;

impl HistoryEntry {
    pub fn read(b: &[u8]) -> Self {
        let b = &b[..HISTORY_LEN];
        Self {
            first: dword(b, 0),
            player: dword(b, 4),
            nation: dword(b, 8),
            car: dword(b, 0xC),
            group: dword(b, 0x10),
            name: b[0x14..0x34].try_into().expect("the name's room"),
            total: dword(b, 0x34),
            overall: std::array::from_fn(|k| dword(b, 0x38 + 4 * k)),
            stage: std::array::from_fn(|k| dword(b, 0x54 + 4 * k)),
            position: dword(b, 0x70),
            status: dword(b, 0x74),
        }
    }

    pub fn write(&self, b: &mut [u8]) {
        let b = &mut b[..HISTORY_LEN];
        put(b, 0, self.first);
        put(b, 4, self.player);
        put(b, 8, self.nation);
        put(b, 0xC, self.car);
        put(b, 0x10, self.group);
        b[0x14..0x34].copy_from_slice(&self.name);
        put(b, 0x34, self.total);
        for k in 0..7 {
            put(b, 0x38 + 4 * k, self.overall[k]);
            put(b, 0x54 + 4 * k, self.stage[k]);
        }
        put(b, 0x70, self.position);
        put(b, 0x74, self.status);
    }

    /// The dword at `offset` (the finishing steps' racer fields, read by offset).
    pub fn dword_at(&self, offset: u32) -> Option<u32> {
        let mut b = [0; HISTORY_LEN];
        self.write(&mut b);
        let at = offset as usize;
        b.get(at..at + 4).map(|d| dword(d, 0))
    }

    /// The dword at `offset` set to `v`; None past the entry.
    pub fn set_dword_at(&mut self, offset: u32, v: u32) -> Option<()> {
        let mut b = [0; HISTORY_LEN];
        self.write(&mut b);
        let at = offset as usize;
        b.get_mut(at..at + 4)?.copy_from_slice(&v.to_le_bytes());
        *self = Self::read(&b);
        Some(())
    }

    /// The text the game draws for the name: from +0x14 to the first NUL. None: no NUL
    /// before the entry's end.
    pub fn name_text(&self) -> Option<Vec<u8>> {
        let mut b = [0; HISTORY_LEN];
        self.write(&mut b);
        let text = &b[0x14..];
        Some(text[..text.iter().position(|&c| c == 0)?].to_vec())
    }
}

impl Walked for HistoryEntry {
    fn first(&self) -> u32 {
        self.first
    }
}

/// Table B.
pub type ChampHistory = Table<HistoryEntry>;

/// `bytes` written into an entry from `at` on (a name with its NUL, which may run past its
/// room as the original's copy does). None: past the entry.
pub fn patch_time(e: &mut BestTime, at: usize, bytes: &[u8]) -> Option<()> {
    let mut b = [0; TIME_LEN];
    e.write(&mut b);
    b.get_mut(at..at + bytes.len())?.copy_from_slice(bytes);
    *e = BestTime::read(&b);
    Some(())
}

/// As `patch_time`, in a history entry.
pub fn patch_history(e: &mut HistoryEntry, at: usize, bytes: &[u8]) -> Option<()> {
    let mut b = [0; HISTORY_LEN];
    e.write(&mut b);
    b.get_mut(at..at + bytes.len())?.copy_from_slice(bytes);
    *e = HistoryEntry::read(&b);
    Some(())
}

/// A row of the summary (0x50 bytes): a rival for the race.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rival {
    /// +0: the name's string id.
    pub name_id: u32,
    /// +4: the nationality.
    pub nation: u32,
    /// +8: the car.
    pub car: u32,
    /// +0xC: the time.
    pub time: u32,
    /// +0x10: the sixteen checkpoint times.
    pub checkpoints: [u32; 16],
}

impl Rival {
    /// Its twenty dwords, as the race reads a row by offset.
    pub fn dwords(&self) -> [u32; 20] {
        let mut d = [0; 20];
        d[..4].copy_from_slice(&[self.name_id, self.nation, self.car, self.time]);
        d[4..].copy_from_slice(&self.checkpoints);
        d
    }

    pub fn from_dwords(d: [u32; 20]) -> Self {
        Self {
            name_id: d[0],
            nation: d[1],
            car: d[2],
            time: d[3],
            checkpoints: d[4..].try_into().expect("sixteen"),
        }
    }
}

pub const SUMMARY_ROWS: usize = 0x5A;
pub const SUMMARY_LEN: usize = SUMMARY_ROWS * 0x50;

/// The summary (0x1C20 bytes).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Summary {
    pub rows: [Rival; SUMMARY_ROWS],
}

impl Default for Summary {
    fn default() -> Self {
        Self {
            rows: [Rival::default(); SUMMARY_ROWS],
        }
    }
}

impl crate::saves::tables::FileTable for Summary {
    fn zeroed(bytes: usize) -> Option<Self> {
        (bytes == SUMMARY_LEN).then(Self::default)
    }

    fn read(b: &[u8]) -> Self {
        Self {
            rows: std::array::from_fn(|n| {
                Rival::from_dwords(std::array::from_fn(|k| dword(b, n * 0x50 + 4 * k)))
            }),
        }
    }

    fn write(&self) -> Vec<u8> {
        let mut b = vec![0; SUMMARY_LEN];
        for (n, r) in self.rows.iter().enumerate() {
            for (k, d) in r.dwords().into_iter().enumerate() {
                put(&mut b, n * 0x50 + 4 * k, d);
            }
        }
        b
    }
}

/// Table A's size: `ROOM` best times and the last dword.
pub const RANKED_LEN: usize = ROOM * TIME_LEN + 4;
/// Table B's.
pub const HISTORY_TABLE_LEN: usize = ROOM * HISTORY_LEN + 4;

impl crate::saves::tables::FileTable for RankedTimes {
    fn zeroed(bytes: usize) -> Option<Self> {
        (bytes == RANKED_LEN).then(Self::new)
    }

    fn read(b: &[u8]) -> Self {
        Self {
            entries: (0..ROOM)
                .map(|n| BestTime::read(&b[n * TIME_LEN..]))
                .collect(),
            end: dword(b, ROOM * TIME_LEN),
        }
    }

    fn write(&self) -> Vec<u8> {
        let mut b = vec![0; RANKED_LEN];
        for (n, e) in self.entries.iter().enumerate() {
            e.write(&mut b[n * TIME_LEN..]);
        }
        put(&mut b, ROOM * TIME_LEN, self.end);
        b
    }
}

impl crate::saves::tables::FileTable for ChampHistory {
    fn zeroed(bytes: usize) -> Option<Self> {
        (bytes == HISTORY_TABLE_LEN).then(Self::new)
    }

    fn read(b: &[u8]) -> Self {
        Self {
            entries: (0..ROOM)
                .map(|n| HistoryEntry::read(&b[n * HISTORY_LEN..]))
                .collect(),
            end: dword(b, ROOM * HISTORY_LEN),
        }
    }

    fn write(&self) -> Vec<u8> {
        let mut b = vec![0; HISTORY_TABLE_LEN];
        for (n, e) in self.entries.iter().enumerate() {
            e.write(&mut b[n * HISTORY_LEN..]);
        }
        put(&mut b, ROOM * HISTORY_LEN, self.end);
        b
    }
}

/// The original's exchange sort over a table's first `count` entries: for each i < j,
/// entry j and i exchanged (j first) when j's key is at most i's (`down` false) or at
/// least i's (`down`), signed.
pub fn exchange_sort<T>(
    list: &mut [T],
    count: usize,
    key: impl Fn(&T) -> u32,
    down: bool,
    mut exchange: impl FnMut(&mut [T], usize, usize),
) {
    for i in 0..count - 1 {
        for j in i + 1..count {
            let (x, y) = (key(&list[j]) as i32, key(&list[i]) as i32);
            if if down { x >= y } else { x <= y } {
                exchange(list, j, i);
            }
        }
    }
}

/// The fields a sort of table A moves (its name too).
pub const A_FIELDS: [Field; 9] = [
    Field::NameId,
    Field::Key,
    Field::Player,
    Field::First,
    Field::Nation,
    Field::Group,
    Field::Car,
    Field::RallyTime,
    Field::Entered,
];

/// Table A's first `count` entries sorted by `key` (an exchange moves `A_FIELDS` and the
/// name, through `kept`).
pub fn sort_a(t: &mut RankedTimes, count: usize, key: Field, kept: &mut [u8; NAME_MOVED]) {
    let key_of = |e: &BestTime| e.head.value(key);
    exchange_sort(&mut t.entries, count, key_of, false, |l, a, b| {
        crate::saves::entries::exchange(l, a, b, &A_FIELDS, kept);
    });
}

/// A key of table B's sorts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BKey {
    /// +0x34.
    Total,
    /// +0x50.
    OverallTotal,
    /// +0x6C.
    StageTotal,
}

/// Entries `a` and `b` of table B exchanged as the original does it: +0x34, +4, +0, +8,
/// +0x10, +0xC, +0x38 .. +0x6C and the name's 0x1F bytes (through `kept`, which ends
/// with `a`'s); the position and the status stay.
pub fn exchange_b(list: &mut [HistoryEntry], a: usize, b: usize, kept: &mut [u8; NAME_MOVED]) {
    let (x, y) = (list[a], list[b]);
    for (to, from) in [(a, y), (b, x)] {
        let e = &mut list[to];
        e.total = from.total;
        e.player = from.player;
        e.first = from.first;
        e.nation = from.nation;
        e.group = from.group;
        e.car = from.car;
        e.overall = from.overall;
        e.stage = from.stage;
        e.name[..NAME_MOVED].copy_from_slice(&from.name[..NAME_MOVED]);
    }
    kept.copy_from_slice(&x.name[..NAME_MOVED]);
}

/// Table B's first `count` entries sorted by `key`, going up or (`down`) down.
pub fn sort_b(
    t: &mut ChampHistory,
    count: usize,
    key: BKey,
    down: bool,
    kept: &mut [u8; NAME_MOVED],
) {
    let key_of = |e: &HistoryEntry| match key {
        BKey::Total => e.total,
        BKey::OverallTotal => e.overall[6],
        BKey::StageTotal => e.stage[6],
    };
    exchange_sort(&mut t.entries, count, key_of, down, |l, a, b| {
        exchange_b(l, a, b, kept);
    });
}

/// What a first-player search finds: the entry, the computer drivers of the group before
/// it, its figure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct First {
    pub entry: usize,
    pub computers: u32,
    pub figure: u32,
}

/// The original's search for the first player (`find1stplayer`) over `walked` entries:
/// those of `group` (`ANY_GROUP`: any whose group is not 0) whose player is not 0; each
/// entry's (group, player, figure) from `fields`. None: the game's fatal error.
pub fn first_player<T>(
    list: &[T],
    group: u32,
    fields: impl Fn(&T) -> (u32, u32, u32),
) -> Option<First> {
    let mut computers = 0;
    for (n, e) in list.iter().enumerate() {
        let (g, player, figure) = fields(e);
        let counts = if group == crate::menu::standings::ANY_GROUP {
            g != 0
        } else {
            g == group
        };
        if !counts {
            continue;
        }
        if player >= 1 {
            return Some(First {
                entry: n,
                computers,
                figure,
            });
        }
        computers += 1;
    }
    None
}

/// Table A's emptied entry (`menu::init_standings_a`): its first eight dwords.
pub fn empty_a(e: &mut BestTime) {
    let h = &mut e.head;
    h.first = u32::MAX;
    h.player = u32::MAX;
    h.nation = 0;
    h.name_id = 0;
    h.car = 0;
    h.key = crate::menu::standings::NO_TIME;
    h.rally_time = crate::menu::standings::NO_TIME;
    h.group = 0;
}

/// Table B's emptied entry: its first five dwords -1.
pub fn empty_b(e: &mut HistoryEntry) {
    e.first = u32::MAX;
    e.player = u32::MAX;
    e.nation = u32::MAX;
    e.car = u32::MAX;
    e.group = u32::MAX;
}

/// A table A entry filled from a player (`menu::copy_standing_entry`).
pub fn fill_a(e: &mut BestTime, p: &crate::menu::standings::PlayerA) {
    let h = &mut e.head;
    h.player = p.index.wrapping_add(1);
    h.key = p.time;
    h.group = p.group;
    h.rally_time = p.total;
    h.name_id = p.name_at;
    h.name = p.name;
    h.nation = p.nationality;
    h.car = p.car;
    h.entered = p.entered;
}

/// A table B entry filled from a player (`menu::player_to_standing_b`).
pub fn fill_b(e: &mut HistoryEntry, p: &crate::menu::standings::PlayerB) {
    e.player = p.index.wrapping_add(1);
    e.name = p.name;
    e.nation = p.nationality;
    e.car = p.car;
    e.group = p.group;
    e.total = p.total;
    e.overall = p.b4;
    e.stage = p.slots;
}

/// A driver's fields, as `fill_a_from_driver` / `fill_b_from_driver` take them: its +0,
/// +8, +0xC, +0x10, +0x14, +0x18, +0x24, and +0x5C .. +0x90 (fourteen dwords).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Driver {
    pub number: u32,
    pub group: u32,
    pub time: u32,
    pub total: u32,
    pub nation: u32,
    pub car: u32,
    pub entered: u32,
    pub points: [u32; 14],
}

/// Table A's entry from a driver (`menu::standings_from_drivers`): its time, the player 0,
/// group, total, nationality, car, number, taking part, the name's string id, and the
/// text of the name (with its NUL) at the name. None: the text past the entry.
pub fn fill_a_from_driver(e: &mut BestTime, d: &Driver, name_id: u32, text: &[u8]) -> Option<()> {
    let h = &mut e.head;
    h.key = d.time;
    h.player = 0;
    h.group = d.group;
    h.rally_time = d.total;
    h.nation = d.nation;
    h.car = d.car;
    h.first = d.number;
    h.entered = d.entered;
    h.name_id = name_id;
    patch_time(e, 0x20, text)
}

/// Table B's entry from a driver (`menu::standings_b_from_all`). None: the text past the
/// entry.
pub fn fill_b_from_driver(e: &mut HistoryEntry, d: &Driver, text: &[u8]) -> Option<()> {
    e.player = 0;
    e.group = d.group;
    e.nation = d.nation;
    e.car = d.car;
    e.first = d.number;
    e.total = d.total;
    e.overall = d.points[..7].try_into().expect("seven");
    e.stage = d.points[7..].try_into().expect("seven");
    patch_history(e, 0x14, text)
}

/// The summary's rows (`menu::copy_standings_summary`) from table A's walked entries with
/// a group (`grouped`) or without: each its name id, nationality, car and stage time;
/// None when they run out first (the original walks on past the table's end).
pub fn summary_rows(walked: &[BestTime], grouped: bool) -> Option<Vec<[u32; 4]>> {
    let rows: Vec<[u32; 4]> = walked
        .iter()
        .filter(|e| (e.head.group != 0) == grouped)
        .take(SUMMARY_ROWS)
        .map(|e| [e.head.name_id, e.head.nation, e.head.car, e.head.key])
        .collect();
    (rows.len() == SUMMARY_ROWS).then_some(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::saves::tables::FileTable;

    #[test]
    fn tables_round_trip_and_end_with_their_mark() {
        let a = RankedTimes::zeroed(RANKED_LEN).unwrap();
        assert_eq!(a.write().len(), 0x4C2C4);
        assert_eq!(a.first_at(ROOM), Some(u32::MAX));
        assert_eq!(a.first_at(ROOM + 1), None);
        let b = ChampHistory::zeroed(HISTORY_TABLE_LEN).unwrap();
        assert_eq!(b.write().len(), 0x3A984);
        let bytes: Vec<u8> = (0..HISTORY_LEN).map(|n| (n * 3) as u8).collect();
        let e = HistoryEntry::read(&bytes);
        let mut out = vec![0; HISTORY_LEN];
        e.write(&mut out);
        assert_eq!(out, bytes);
        assert_eq!(e.dword_at(0x6C), Some(e.stage[6]));
        assert_eq!(Summary::zeroed(0x1C20).unwrap().write().len(), 0x1C20);
    }

    #[test]
    fn walks_stop_at_an_end_entry() {
        let mut a = RankedTimes::zeroed(RANKED_LEN).unwrap();
        assert_eq!(a.walked(), None, "no end in 0x400");
        a.entries[3].head.first = u32::MAX;
        assert_eq!(a.walked(), Some(3));
        a.entries[0].head.first = u32::MAX;
        assert_eq!(a.walked(), Some(3), "the first is always walked");
    }

    #[test]
    fn sorting_b_moves_all_but_the_position() {
        let mut b = ChampHistory::zeroed(HISTORY_TABLE_LEN).unwrap();
        for (n, e) in b.entries.iter_mut().take(3).enumerate() {
            e.total = n as u32;
            e.position = 10 + n as u32;
            e.name = [b'a' + n as u8; 0x20];
        }
        let mut kept = [0; NAME_MOVED];
        sort_b(&mut b, 3, BKey::Total, true, &mut kept);
        assert_eq!(
            [b.entries[0].total, b.entries[1].total, b.entries[2].total],
            [2, 1, 0]
        );
        assert_eq!(b.entries[0].position, 10);
        assert_eq!(b.entries[0].name[0], b'c');
    }

    /// The tables' first `n` entries from varied bytes, as typed and as bytes.
    fn varied(len: usize, n: usize, seed: usize) -> Vec<u8> {
        (0..len * n)
            .map(|k| ((k * seed + k / 7) % 251) as u8)
            .collect()
    }

    #[test]
    fn sorts_match_the_byte_versions() {
        use crate::menu::standings::{self as bytes, Entries};
        let count = bytes::SORTED;
        for (key_at, key) in [(0x14, Field::Key), (0x18, Field::RallyTime)] {
            let mut raw = varied(TIME_LEN, count, 31);
            let mut a = RankedTimes::zeroed(RANKED_LEN).unwrap();
            for n in 0..count {
                a.entries[n] = BestTime::read(&raw[n * TIME_LEN..]);
            }
            let (mut k1, mut k2) = ([0; NAME_MOVED], [0; NAME_MOVED]);
            bytes::exchange_sort(
                &mut Entries {
                    bytes: &mut raw,
                    size: TIME_LEN,
                },
                key_at,
                false,
                &bytes::A_FIELDS,
                bytes::A_NAME,
                &mut k1,
            );
            sort_a(&mut a, count, key, &mut k2);
            let mut out = vec![0; TIME_LEN * count];
            for n in 0..count {
                a.entries[n].write(&mut out[n * TIME_LEN..]);
            }
            assert_eq!(out, raw);
            assert_eq!(k1, k2);
        }
        for (key_at, key, down) in [
            (0x34, BKey::Total, false),
            (0x50, BKey::OverallTotal, true),
            (0x6C, BKey::StageTotal, true),
        ] {
            let mut raw = varied(HISTORY_LEN, count, 17);
            let mut b = ChampHistory::zeroed(HISTORY_TABLE_LEN).unwrap();
            for n in 0..count {
                b.entries[n] = HistoryEntry::read(&raw[n * HISTORY_LEN..]);
            }
            let (mut k1, mut k2) = ([0; NAME_MOVED], [0; NAME_MOVED]);
            bytes::exchange_sort(
                &mut Entries {
                    bytes: &mut raw,
                    size: HISTORY_LEN,
                },
                key_at,
                down,
                &bytes::b_fields(),
                bytes::B_NAME,
                &mut k1,
            );
            sort_b(&mut b, count, key, down, &mut k2);
            let mut out = vec![0; HISTORY_LEN * count];
            for n in 0..count {
                b.entries[n].write(&mut out[n * HISTORY_LEN..]);
            }
            assert_eq!(out, raw);
            assert_eq!(k1, k2);
        }
    }

    #[test]
    fn fills_match_the_byte_versions() {
        use crate::menu::standings::{self as bytes, Entries, PlayerA, PlayerB};
        let mut raw = varied(TIME_LEN, 1, 7);
        let mut e = BestTime::read(&raw);
        let pa = PlayerA {
            index: 2,
            time: 3,
            group: 4,
            total: 5,
            name_at: 6,
            name: [b'n'; 0x20],
            nationality: 7,
            car: 8,
            entered: 9,
        };
        bytes::fill_a(
            &mut Entries {
                bytes: &mut raw,
                size: TIME_LEN,
            },
            0,
            &pa,
        );
        fill_a(&mut e, &pa);
        let mut out = vec![0; TIME_LEN];
        e.write(&mut out);
        assert_eq!(out, raw);
        bytes::empty_bytes_a(&mut raw);
        empty_a(&mut e);
        e.write(&mut out);
        assert_eq!(out, raw);

        let mut raw = varied(HISTORY_LEN, 1, 5);
        let mut h = HistoryEntry::read(&raw);
        let pb = PlayerB {
            index: 1,
            name: [b'm'; 0x20],
            nationality: 2,
            car: 3,
            group: 4,
            total: 5,
            b4: [6; 7],
            slots: [7; 7],
        };
        bytes::fill_b(
            &mut Entries {
                bytes: &mut raw,
                size: HISTORY_LEN,
            },
            0,
            &pb,
        );
        fill_b(&mut h, &pb);
        let mut out = vec![0; HISTORY_LEN];
        h.write(&mut out);
        assert_eq!(out, raw);

        // From a driver: the driver's record as bytes for the byte version.
        let mut drv = varied(0x94, 1, 11);
        let d = Driver {
            number: dword(&drv, 0),
            group: dword(&drv, 8),
            time: dword(&drv, 0xC),
            total: dword(&drv, 0x10),
            nation: dword(&drv, 0x14),
            car: dword(&drv, 0x18),
            entered: dword(&drv, 0x24),
            points: std::array::from_fn(|k| dword(&drv, 0x5C + 4 * k)),
        };
        let drivers = Entries {
            bytes: &mut drv,
            size: 0x94,
        };
        let text = b"A driver\0";
        let mut raw = varied(TIME_LEN, 1, 3);
        let mut e = BestTime::read(&raw);
        bytes::fill_a_from_driver(
            &mut Entries {
                bytes: &mut raw,
                size: TIME_LEN,
            },
            0,
            &drivers,
            0,
            0x55,
            text,
        );
        fill_a_from_driver(&mut e, &d, 0x55, text).unwrap();
        let mut out = vec![0; TIME_LEN];
        e.write(&mut out);
        assert_eq!(out, raw);
        let mut raw = varied(HISTORY_LEN, 1, 3);
        let mut h = HistoryEntry::read(&raw);
        bytes::fill_b_from_driver(
            &mut Entries {
                bytes: &mut raw,
                size: HISTORY_LEN,
            },
            0,
            &drivers,
            0,
            text,
        );
        fill_b_from_driver(&mut h, &d, text).unwrap();
        let mut out = vec![0; HISTORY_LEN];
        h.write(&mut out);
        assert_eq!(out, raw);
    }
}
