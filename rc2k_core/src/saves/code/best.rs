//! The best times in the code: only the entries a player made. The made-up ones the game fills
//! its lists with (a driver of the game's own, with a time to match) are not carried; the
//! importing game keeps its own and the player's entries are ranked in among them as a new
//! best time is.
//!
//! An entry is the player's when its player number is not 0 or its name is not one of the
//! game's drivers' (the best scores clear the number once their page has been shown).

use super::CodeError;
use crate::saves::entries::{
    BestTime, ENTRY_LEN, Entry, Field, NAME_MOVED, PER_LIST, RECORD_FIELDS, Ranked, SCORE_FIELDS,
    TIME_FIELDS, TIME_LEN,
};
use crate::saves::scores::{ENTRY_LEN as SCORE_LEN, NAME_MOVED as SCORE_NAME_MOVED, ScoreEntry};

/// The names of the game's drivers, which the made-up entries carry.
#[derive(Clone, Debug, Default)]
pub struct DriverNames(pub Vec<Vec<u8>>);

fn until_nul(name: &[u8]) -> &[u8] {
    &name[..name.iter().position(|&c| c == 0).unwrap_or(name.len())]
}

impl DriverNames {
    fn has(&self, name: &[u8]) -> bool {
        let name = until_nul(name);
        name.is_empty() || self.0.iter().any(|d| d == name)
    }
}

/// What the lists' code needs of an entry.
trait Item: Copy {
    fn read(b: &[u8]) -> Self;
    fn write(&self, b: &mut [u8]);
    fn name(&self) -> &[u8];
    fn player(&self) -> u32;
    /// What ranks it, as the rule's `key` names it.
    fn key(&self, by: By) -> u32;
    /// The fields a ranking moves, from `from`.
    fn take(&mut self, from: &Self, fields: &[Field]);
    fn made_up(&self, names: &DriverNames) -> bool {
        self.player() == 0 && names.has(&self.name()[..0x20])
    }
}

impl Item for BestTime {
    fn read(b: &[u8]) -> Self {
        BestTime::read(b)
    }
    fn write(&self, b: &mut [u8]) {
        BestTime::write(self, b);
    }
    fn name(&self) -> &[u8] {
        &self.head.name
    }
    fn player(&self) -> u32 {
        self.head.player
    }
    fn key(&self, by: By) -> u32 {
        match by {
            By::Key => self.head.key,
            By::RallyTime => self.head.rally_time,
        }
    }
    fn take(&mut self, from: &Self, fields: &[Field]) {
        let (mut d, mut s) = (*self, *from);
        for &f in fields {
            BestTime::exchange_field(&mut d, &mut s, f);
        }
        d.head.name[..NAME_MOVED].copy_from_slice(&from.head.name[..NAME_MOVED]);
        *self = d;
    }
}

impl Item for Entry {
    fn read(b: &[u8]) -> Self {
        Entry::read(b)
    }
    fn write(&self, b: &mut [u8]) {
        Entry::write(self, b);
    }
    fn name(&self) -> &[u8] {
        &self.name
    }
    fn player(&self) -> u32 {
        self.player
    }
    fn key(&self, _: By) -> u32 {
        self.key
    }
    fn take(&mut self, from: &Self, fields: &[Field]) {
        let (mut d, mut s) = (*self, *from);
        for &f in fields {
            Entry::exchange_field(&mut d, &mut s, f);
        }
        d.name[..NAME_MOVED].copy_from_slice(&from.name[..NAME_MOVED]);
        *self = d;
    }
}

impl Item for ScoreEntry {
    fn read(b: &[u8]) -> Self {
        ScoreEntry::read(b.try_into().expect("an entry"))
    }
    fn write(&self, b: &mut [u8]) {
        ScoreEntry::write(self, b.try_into().expect("an entry"));
    }
    fn name(&self) -> &[u8] {
        &self.name
    }
    fn player(&self) -> u32 {
        self.player
    }
    fn key(&self, _: By) -> u32 {
        self.score
    }
    fn take(&mut self, from: &Self, _: &[Field]) {
        // What a ranking of the scores moves (not the name's place or its last byte).
        self.score = from.score;
        self.new = from.new;
        self.player = from.player;
        self.nation = from.nation;
        self.group = from.group;
        self.car = from.car;
        self.name[..SCORE_NAME_MOVED].copy_from_slice(&from.name[..SCORE_NAME_MOVED]);
    }
}

#[derive(Clone, Copy)]
enum By {
    Key,
    RallyTime,
}

/// How a list of ten is ordered, as the game's ranking does it.
#[derive(Clone, Copy)]
struct Rule {
    fields: &'static [Field],
    by: By,
    /// The best is the highest.
    descending: bool,
    /// Keys compare as signed numbers.
    signed: bool,
    /// An entry that ties with another goes above it (the game moves a new entry up past
    /// equals in the rules that compare with "at most").
    new_above_equals: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Score,
    Level,
    Time,
}

/// A table of `var\hst.dat` that holds lists of ten.
struct Table {
    kind: Kind,
    entry_len: usize,
    rule: Rule,
}

const RECORDS: Rule = Rule {
    fields: &RECORD_FIELDS,
    by: By::Key,
    descending: false,
    signed: true,
    new_above_equals: true,
};
const ARCADE: Rule = Rule {
    fields: &TIME_FIELDS,
    by: By::Key,
    descending: false,
    signed: true,
    new_above_equals: true,
};
const TRIAL: Rule = Rule {
    fields: &TIME_FIELDS,
    by: By::Key,
    descending: false,
    signed: false,
    new_above_equals: false,
};
const LEVELS: Rule = Rule {
    fields: &SCORE_FIELDS,
    by: By::Key,
    descending: true,
    signed: true,
    new_above_equals: true,
};
const RALLY: Rule = Rule {
    fields: &TIME_FIELDS,
    by: By::RallyTime,
    descending: false,
    signed: true,
    new_above_equals: true,
};

/// The six tables the file holds after its head, in the file's order: the best scores, the
/// stages' record entries, the levels' scores, the courses' times, the other group's stage
/// times and the time trial's.
const TABLES: [Table; 6] = [
    Table {
        kind: Kind::Score,
        entry_len: SCORE_LEN,
        rule: LEVELS,
    },
    Table {
        kind: Kind::Time,
        entry_len: TIME_LEN,
        rule: RECORDS,
    },
    Table {
        kind: Kind::Level,
        entry_len: ENTRY_LEN,
        rule: LEVELS,
    },
    Table {
        kind: Kind::Time,
        entry_len: TIME_LEN,
        rule: ARCADE,
    },
    Table {
        kind: Kind::Time,
        entry_len: TIME_LEN,
        rule: TRIAL,
    },
    Table {
        kind: Kind::Time,
        entry_len: TIME_LEN,
        rule: TRIAL,
    },
];

/// How many lists a table can hold at most (the championship has 36 stages).
const MOST_LISTS: usize = 64;

/// An entry a player made, in list `list` of table `table` (0 .. 6, the file's order).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Real {
    pub table: u8,
    pub list: u16,
    pub entry: Vec<u8>,
}

/// A rally's best time a player made (the options file's tables).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RallyTime {
    pub rally: u8,
    /// 0 or 1: a rally's table holds two lists of ten.
    pub list: u8,
    pub entry: Vec<u8>,
}

/// A stage record held by a player: the record's place and the parts a holder changes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StageRecord {
    /// 0 the championship's table, 1 an event's.
    pub group: u8,
    pub event: u8,
    pub index: u16,
    pub bytes: Vec<u8>,
}

/// The stage records' length.
pub const RECORD_LEN: usize = 0xA4;
/// The parts of a stage record a new holder changes: the class times and speeds, and both
/// holders.
const RECORD_PARTS: [std::ops::Range<usize>; 2] = [0x18..0x34, 0x54..RECORD_LEN];
const RECORD_TAG: usize = 0;
const RECORD_TRACK: usize = 4;
const MOST_RECORDS: usize = 256;
const EVENTS: usize = 12;
const RALLIES: usize = 6;

fn lists_of<T: Item>(bytes: &[u8], entry_len: usize, mut each: impl FnMut(usize, usize, T)) {
    for (n, chunk) in bytes.chunks_exact(entry_len).enumerate() {
        each(n / PER_LIST, n % PER_LIST, T::read(chunk));
    }
}

/// The entries the player made in the tables `tables` (the file's six, in its order).
pub fn extract(tables: &[Vec<u8>; 6], names: &DriverNames) -> Vec<Real> {
    let mut out = Vec::new();
    for (t, (table, bytes)) in TABLES.iter().zip(tables).enumerate() {
        let mut push = |list: usize, entry: &[u8], made_up: bool| {
            if !made_up {
                out.push(Real {
                    table: t as u8,
                    list: list as u16,
                    entry: entry.to_vec(),
                });
            }
        };
        for (n, chunk) in bytes.chunks_exact(table.entry_len).enumerate() {
            let made_up = match table.kind {
                Kind::Score => ScoreEntry::read(chunk.try_into().expect("an entry")).made_up(names),
                Kind::Level => Entry::read(chunk).made_up(names),
                Kind::Time => BestTime::read(chunk).made_up(names),
            };
            push(n / PER_LIST, chunk, made_up);
        }
    }
    out
}

fn merge_list<T: Item>(slots: &mut [T], real: &[T], rule: Rule, names: &DriverNames) {
    // A list with nothing of a player's in it, here or there, is left as it is.
    if real.is_empty() && slots.iter().all(|s| s.made_up(names)) {
        return;
    }
    let mut sorted: Vec<(T, bool)> = real.iter().map(|&e| (e, true)).collect();
    let mut kept = Vec::new();
    for &s in slots.iter() {
        if s.made_up(names) {
            sorted.push((s, false));
        } else {
            kept.push(s);
        }
    }
    let order = |&(e, is_real): &(T, bool)| {
        let k = e.key(rule.by);
        let k = if rule.signed {
            i64::from(k as i32)
        } else {
            i64::from(k)
        };
        let k = if rule.descending { -k } else { k };
        (k, is_real != rule.new_above_equals)
    };
    sorted.sort_by_key(order);
    sorted.truncate(slots.len());
    // A list that came short (the importing game's own entries were the player's) keeps them.
    let mut rest = kept.into_iter();
    let mut ranked: Vec<T> = sorted.into_iter().map(|(e, _)| e).collect();
    while ranked.len() < slots.len() {
        match rest.next() {
            Some(e) => ranked.push(e),
            None => break,
        }
    }
    for (slot, e) in slots.iter_mut().zip(&ranked) {
        slot.take(e, rule.fields);
    }
}

fn merge_bytes<T: Item>(
    bytes: &mut [u8],
    entry_len: usize,
    list: usize,
    real: &[T],
    rule: Rule,
    names: &DriverNames,
) {
    let at = list * PER_LIST * entry_len;
    let part = &mut bytes[at..at + PER_LIST * entry_len];
    let mut slots: Vec<T> = part.chunks_exact(entry_len).map(T::read).collect();
    merge_list(&mut slots, real, rule, names);
    for (chunk, e) in part.chunks_exact_mut(entry_len).zip(&slots) {
        e.write(chunk);
    }
}

/// `tables` with the player's `real` entries ranked into the lists over the made-up ones;
/// the other entries of the importing game's player are dropped. Refused: an entry of a list
/// the tables do not have.
pub fn merge(
    tables: &mut [Vec<u8>; 6],
    real: &[Real],
    names: &DriverNames,
) -> Result<(), CodeError> {
    for (t, (table, bytes)) in TABLES.iter().zip(tables.iter_mut()).enumerate() {
        let lists = bytes.len() / (PER_LIST * table.entry_len);
        let mut wanted: Vec<usize> = real
            .iter()
            .filter(|r| usize::from(r.table) == t)
            .map(|r| usize::from(r.list))
            .collect();
        wanted.sort_unstable();
        wanted.dedup();
        if wanted.last().is_some_and(|&l| l >= lists) {
            return Err(CodeError::Range(
                "a best time of a list this game does not have",
            ));
        }
        // Every list of the table is made over, those the player has nothing in as well: the
        // importing game's own entries of the player's are not the imported player's.
        for list in 0..lists {
            let mine = real
                .iter()
                .filter(|r| usize::from(r.table) == t && usize::from(r.list) == list);
            match table.kind {
                Kind::Score => {
                    let e: Vec<ScoreEntry> =
                        mine.map(|r| <ScoreEntry as Item>::read(&r.entry)).collect();
                    merge_bytes(bytes, table.entry_len, list, &e, table.rule, names);
                }
                Kind::Level => {
                    let e: Vec<Entry> = mine.map(|r| Entry::read(&r.entry)).collect();
                    merge_bytes(bytes, table.entry_len, list, &e, table.rule, names);
                }
                Kind::Time => {
                    let e: Vec<BestTime> = mine.map(|r| BestTime::read(&r.entry)).collect();
                    merge_bytes(bytes, table.entry_len, list, &e, table.rule, names);
                }
            }
        }
    }
    Ok(())
}

/// Whether the entry is one a player could have made, as far as its numbers say.
fn entry_ok(kind: Kind, entry: &[u8]) -> bool {
    let word = |at: usize| u32::from_le_bytes(entry[at..at + 4].try_into().expect("a dword"));
    let (player, nation, group, car, new_at, name_at) = match kind {
        Kind::Score => (word(4), word(8), word(0x10), word(0xC), 0x14, 0x18),
        Kind::Level | Kind::Time => (word(4), word(8), word(0x1C), word(0x10), 0x40, 0x20),
    };
    let name = &entry[name_at..name_at + 0x20];
    player <= 4
        && nation < 0x100
        && group < 0x100
        && car < 0x400
        && word(new_at) <= 1
        && name.iter().take_while(|&&b| b != 0).all(|&b| b >= 0x20)
}

/// Checks what a code says of the best times before anything is done with it.
pub fn check(real: &[Real]) -> Result<(), CodeError> {
    for r in real {
        let table = TABLES
            .get(usize::from(r.table))
            .ok_or(CodeError::Range("a best time of a table there is not"))?;
        if r.entry.len() != table.entry_len || usize::from(r.list) >= MOST_LISTS {
            return Err(CodeError::Range("a best time of the wrong size or list"));
        }
        if !entry_ok(table.kind, &r.entry) {
            return Err(CodeError::Range(
                "a best time with a value out of its range",
            ));
        }
    }
    Ok(())
}

/// The rally times the player made in the options' rally tables (0xC30 bytes each).
pub fn extract_rallies(rallies: &[&[u8]], names: &DriverNames) -> Vec<RallyTime> {
    let mut out = Vec::new();
    for (rally, bytes) in rallies.iter().enumerate() {
        lists_of::<BestTime>(bytes, TIME_LEN, |list, _, e| {
            if !e.made_up(names) {
                let mut entry = vec![0; TIME_LEN];
                e.write(&mut entry);
                out.push(RallyTime {
                    rally: rally as u8,
                    list: list as u8,
                    entry,
                });
            }
        });
    }
    out
}

/// The rallies' tables with the player's `times` ranked over the made-up ones.
pub fn merge_rallies(
    rallies: &mut [&mut [u8]],
    times: &[RallyTime],
    names: &DriverNames,
) -> Result<(), CodeError> {
    for (rally, bytes) in rallies.iter_mut().enumerate() {
        let lists = bytes.len() / (PER_LIST * TIME_LEN);
        for list in 0..lists {
            let e: Vec<BestTime> = times
                .iter()
                .filter(|t| usize::from(t.rally) == rally && usize::from(t.list) == list)
                .map(|t| BestTime::read(&t.entry))
                .collect();
            merge_bytes(bytes, TIME_LEN, list, &e, RALLY, names);
        }
    }
    if times
        .iter()
        .any(|t| usize::from(t.rally) >= rallies.len() || usize::from(t.list) >= 2)
    {
        return Err(CodeError::Range(
            "a rally time of a rally or list there is not",
        ));
    }
    Ok(())
}

pub fn check_rallies(times: &[RallyTime]) -> Result<(), CodeError> {
    for t in times {
        if usize::from(t.rally) >= RALLIES || t.list >= 2 || t.entry.len() != TIME_LEN {
            return Err(CodeError::Range("a rally time of the wrong place or size"));
        }
        if !entry_ok(Kind::Time, &t.entry) {
            return Err(CodeError::Range(
                "a rally time with a value out of its range",
            ));
        }
    }
    Ok(())
}

/// Whether a stage record names one of `players` as a holder.
fn held_by(record: &[u8], players: &[Vec<u8>]) -> bool {
    [&record[0x54..0x74], &record[0x7C..0x9C]]
        .iter()
        .any(|name| {
            let name = until_nul(name);
            !name.is_empty() && players.iter().any(|p| until_nul(p) == name)
        })
}

/// The stage records the players hold: the championship's table (`0xA4`-byte records), then
/// the twelve events' tables.
pub fn extract_records(
    championship: &[u8],
    events: &[&[u8]],
    players: &[Vec<u8>],
) -> Vec<StageRecord> {
    let mut out = Vec::new();
    let tables = std::iter::once((0u8, 0u8, championship))
        .chain(events.iter().enumerate().map(|(e, b)| (1, e as u8, *b)));
    for (group, event, bytes) in tables {
        for (index, record) in bytes.as_chunks::<RECORD_LEN>().0.iter().enumerate() {
            if held_by(record, players) {
                out.push(StageRecord {
                    group,
                    event,
                    index: index as u16,
                    bytes: record.to_vec(),
                });
            }
        }
    }
    out
}

fn record_at<'a>(
    championship: &'a mut [u8],
    events: &'a mut [&mut [u8]],
    r: &StageRecord,
) -> Option<&'a mut [u8]> {
    let table = match r.group {
        0 => championship,
        _ => &mut **events.get_mut(usize::from(r.event))?,
    };
    table
        .as_chunks_mut::<RECORD_LEN>()
        .0
        .get_mut(usize::from(r.index))
        .map(|record| &mut record[..])
}

/// The records `records` laid over those of the tables, a record only where its stage is the
/// same one. The number of records that found their stage.
pub fn merge_records(
    championship: &mut [u8],
    events: &mut [&mut [u8]],
    records: &[StageRecord],
) -> usize {
    let mut placed = 0;
    for r in records {
        let Some(at) = record_at(championship, events, r) else {
            continue;
        };
        if at[RECORD_TAG..RECORD_TAG + 4] != r.bytes[RECORD_TAG..RECORD_TAG + 4]
            || at[RECORD_TRACK..RECORD_TRACK + 4] != r.bytes[RECORD_TRACK..RECORD_TRACK + 4]
        {
            continue;
        }
        for part in &RECORD_PARTS {
            at[part.clone()].copy_from_slice(&r.bytes[part.clone()]);
        }
        placed += 1;
    }
    placed
}

pub fn check_records(records: &[StageRecord]) -> Result<(), CodeError> {
    for r in records {
        if r.group > 1 || usize::from(r.event) >= EVENTS || usize::from(r.index) >= MOST_RECORDS {
            return Err(CodeError::Range("a stage record of a place there is not"));
        }
        if r.bytes.len() != RECORD_LEN {
            return Err(CodeError::Range("a stage record of the wrong size"));
        }
        for at in [0x54, 0x7C] {
            let name = &r.bytes[at..at + 0x20];
            if !name.contains(&0) || name.iter().take_while(|&&b| b != 0).any(|&b| b < 0x20) {
                return Err(CodeError::Range("a record holder's name that is not one"));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names() -> DriverNames {
        DriverNames(vec![b"Colin".to_vec(), b"Tommi".to_vec(), b"Juha".to_vec()])
    }

    fn time_entry(player: u32, name: &[u8], key: u32, rally: bool) -> BestTime {
        let mut e = BestTime::default();
        e.head.player = player;
        e.head.name[..name.len()].copy_from_slice(name);
        if rally {
            e.head.rally_time = key;
        } else {
            e.head.key = key;
        }
        e
    }

    fn table(list: [BestTime; 10]) -> Vec<u8> {
        let mut b = vec![0; PER_LIST * TIME_LEN];
        for (n, e) in list.iter().enumerate() {
            e.write(&mut b[n * TIME_LEN..]);
        }
        b
    }

    fn six(time_trial: Vec<u8>) -> [Vec<u8>; 6] {
        [
            vec![0; 5 * 10 * SCORE_LEN],
            vec![0; 10 * TIME_LEN],
            vec![0; 7 * 10 * ENTRY_LEN],
            vec![0; 10 * TIME_LEN],
            vec![0; 10 * TIME_LEN],
            time_trial,
        ]
    }

    #[test]
    fn only_the_players_entries_are_taken() {
        let list: [BestTime; 10] = std::array::from_fn(|n| {
            if n == 3 {
                time_entry(1, b"Rafael", 5000, false)
            } else {
                time_entry(
                    0,
                    [&b"Colin"[..], b"Tommi", b"Juha"][n % 3],
                    1000 * n as u32,
                    false,
                )
            }
        });
        let real = extract(&six(table(list)), &names());
        assert_eq!(real.len(), 1);
        assert_eq!((real[0].table, real[0].list), (5, 0));
        assert_eq!(BestTime::read(&real[0].entry).head.key, 5000);
        check(&real).unwrap();
    }

    #[test]
    fn a_players_time_is_ranked_in_among_the_other_games_made_up_times() {
        let theirs = time_entry(1, b"Ana", 2500, false);
        let mut entry = vec![0; TIME_LEN];
        theirs.write(&mut entry);
        let real = vec![Real {
            table: 5,
            list: 0,
            entry,
        }];
        // The importing game's own list: made-up times, and a time of its own player.
        let mine: [BestTime; 10] = std::array::from_fn(|n| {
            if n == 0 {
                time_entry(2, b"Me", 100, false)
            } else {
                time_entry(0, b"Colin", 1000 * n as u32, false)
            }
        });
        let mut tables = six(table(mine));
        merge(&mut tables, &real, &names()).unwrap();
        let out = BestTime::read(&tables[5][..TIME_LEN]);
        let keys: Vec<u32> = (0..10)
            .map(|n| BestTime::read(&tables[5][n * TIME_LEN..]).head.key)
            .collect();
        // The game's own player is gone; "Ana" sits between 2000 and 3000.
        assert_eq!(
            keys,
            [1000, 2000, 2500, 3000, 4000, 5000, 6000, 7000, 8000, 9000]
        );
        assert_eq!(out.head.player, 0);
        let ana = BestTime::read(&tables[5][2 * TIME_LEN..]);
        assert_eq!((ana.head.player, &ana.head.name[..3]), (1, &b"Ana"[..]));
    }

    #[test]
    fn a_best_time_of_a_list_that_is_not_there_is_refused() {
        let real = vec![Real {
            table: 5,
            list: 3,
            entry: vec![0; TIME_LEN],
        }];
        assert!(merge(&mut six(vec![0; 10 * TIME_LEN]), &real, &names()).is_err());
    }

    #[test]
    fn out_of_range_entries_are_refused() {
        let ok = Real {
            table: 5,
            list: 0,
            entry: vec![0; TIME_LEN],
        };
        check(std::slice::from_ref(&ok)).unwrap();
        for bad in [
            Real {
                table: 6,
                ..ok.clone()
            },
            Real {
                list: 64,
                ..ok.clone()
            },
            Real {
                entry: vec![0; 5],
                ..ok.clone()
            },
            Real {
                entry: {
                    let mut e = vec![0; TIME_LEN];
                    e[4] = 9;
                    e
                },
                ..ok.clone()
            },
            Real {
                entry: {
                    let mut e = vec![0; TIME_LEN];
                    e[0x20] = 1;
                    e
                },
                ..ok.clone()
            },
            Real {
                entry: {
                    let mut e = vec![0; TIME_LEN];
                    e[0x1C] = 0xFF;
                    e[0x1D] = 1;
                    e
                },
                ..ok.clone()
            },
        ] {
            assert!(check(&[bad]).is_err());
        }
    }

    #[test]
    fn scores_and_levels_rank_the_highest_first() {
        let mut scores = ScoreEntry {
            score: 45,
            ..ScoreEntry::default()
        };
        scores.name[..3].copy_from_slice(b"Ana");
        let mut entry = vec![0; SCORE_LEN];
        scores.write((&mut entry[..]).try_into().unwrap());
        let real = vec![Real {
            table: 0,
            list: 2,
            entry,
        }];
        let mut tables = six(vec![0; 10 * TIME_LEN]);
        for (n, chunk) in tables[0]
            .as_chunks_mut::<SCORE_LEN>()
            .0
            .iter_mut()
            .enumerate()
        {
            let mut e = ScoreEntry {
                score: 50 - (n % 10) as u32,
                ..ScoreEntry::default()
            };
            e.name[..5].copy_from_slice(b"Colin");
            e.write(chunk);
        }
        merge(&mut tables, &real, &names()).unwrap();
        let list: Vec<u32> = (0..10)
            .map(|n| {
                ScoreEntry::read(
                    tables[0][(20 + n) * SCORE_LEN..][..SCORE_LEN]
                        .try_into()
                        .unwrap(),
                )
                .score
            })
            .collect();
        assert_eq!(list, [50, 49, 48, 47, 46, 45, 45, 44, 43, 42]);
        let ana = ScoreEntry::read(
            tables[0][(25) * SCORE_LEN..][..SCORE_LEN]
                .try_into()
                .unwrap(),
        );
        assert_eq!(&ana.name[..3], b"Ana");
    }

    #[test]
    fn stage_records_held_by_a_player_are_carried_to_the_same_stage() {
        let players = vec![b"Ana".to_vec()];
        let record = |track: u8, holder: &[u8]| {
            let mut r = vec![0u8; RECORD_LEN];
            r[4] = track;
            r[0x54..0x54 + holder.len()].copy_from_slice(holder);
            r[0x18] = track + 100;
            r
        };
        let theirs = [record(1, b"Colin"), record(2, b"Ana")].concat();
        let found = extract_records(&theirs, &[], &players);
        assert_eq!(found.len(), 1);
        check_records(&found).unwrap();
        let mut mine = [record(1, b"Tommi"), record(2, b"Juha")].concat();
        assert_eq!(merge_records(&mut mine, &mut [], &found), 1);
        assert_eq!(&mine[RECORD_LEN + 0x54..RECORD_LEN + 0x57], b"Ana");
        assert_eq!(&mine[0x54..0x59], b"Tommi");
        // Another course in that place: not placed.
        let mut other = [record(1, b"Tommi"), record(9, b"Juha")].concat();
        assert_eq!(merge_records(&mut other, &mut [], &found), 0);
    }

    #[test]
    fn rally_times_are_ranked_by_their_own_time() {
        let theirs = time_entry(1, b"Ana", 2500, true);
        let mut entry = vec![0; TIME_LEN];
        theirs.write(&mut entry);
        let times = vec![RallyTime {
            rally: 1,
            list: 0,
            entry,
        }];
        check_rallies(&times).unwrap();
        let mine: [BestTime; 10] =
            std::array::from_fn(|n| time_entry(0, b"Colin", 1000 * (n as u32 + 1), true));
        let mut rallies = vec![vec![0u8; 2 * PER_LIST * TIME_LEN]; 6];
        rallies[1][..PER_LIST * TIME_LEN].copy_from_slice(&table(mine));
        let mut parts: Vec<&mut [u8]> = rallies.iter_mut().map(Vec::as_mut_slice).collect();
        merge_rallies(&mut parts, &times, &names()).unwrap();
        let got = BestTime::read(&rallies[1][2 * TIME_LEN..]);
        assert_eq!((got.head.rally_time, got.head.player), (2500, 1));
    }
}
