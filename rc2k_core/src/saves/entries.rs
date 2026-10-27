//! The best lists' entry (typed 2026-10-27, phase 12). A best time is 0x9C bytes, and its
//! first 0x50, `Entry` here, are laid out as the arcade levels' best scores are whole. The
//! lists are of ten, best first; a new entry goes over the tenth and moves up.
//!
//! The levels' best scores (`g_levelBestScores`, 0x7556DC held its block; 0x15E0 bytes in
//! memory and in `var\hst.dat`): seven groups of ten `Entry`s (the arcade's levels 0 .. 5
//! and the page of `menu::view_best_scores_loop` that shows "class times").

use crate::menu::bests::NewBest;

pub const ENTRY_LEN: usize = 0x50;
pub const PER_LIST: usize = 10;
/// The name's room (+0x20 .. +0x40).
pub const NAME_ROOM: usize = 0x20;
/// What a move takes of a name (and what an entered name gets): its room less the last byte.
pub const NAME_MOVED: usize = 0x1F;

/// The head of a best entry (0x50 bytes).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Entry {
    /// +0: -1 ends a list (`rc2k_core::screens::best_scores`); otherwise not used here.
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
    /// +0x20: the name. A made-up entry's fills what its driver's name needs, without a
    /// NUL; an entered one gets the player's first 0x1F bytes.
    pub name: [u8; NAME_ROOM],
    /// +0x40: 1 for the entry just made.
    pub mark: u32,
    /// +0x44.
    pub figure: u32,
    /// +0x48.
    pub field_48: u32,
    /// +0x4C: the records' taking part.
    pub entered: u32,
}

fn dword(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(b[at..at + 4].try_into().expect("four bytes"))
}

fn put(b: &mut [u8], at: usize, v: u32) {
    b[at..at + 4].copy_from_slice(&v.to_le_bytes());
}

/// A dword field of an entry, for the lists of fields the original moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Player,
    Nation,
    NameId,
    Car,
    Key,
    RallyTime,
    Group,
    Mark,
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
            field_48: dword(b, 0x48),
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
        put(b, 0x48, self.field_48);
        put(b, 0x4C, self.entered);
    }

    fn field(&mut self, f: Field) -> &mut u32 {
        match f {
            Field::Player => &mut self.player,
            Field::Nation => &mut self.nation,
            Field::NameId => &mut self.name_id,
            Field::Car => &mut self.car,
            Field::Key => &mut self.key,
            Field::RallyTime => &mut self.rally_time,
            Field::Group => &mut self.group,
            Field::Mark => &mut self.mark,
        }
    }

    /// The text the game draws for the name: from +0x20 to the first NUL, which may come
    /// after the name's room (the original reads on into the mark). None: no NUL before
    /// the head's end.
    pub fn name_text(&self) -> Option<Vec<u8>> {
        let mut bytes = self.name.to_vec();
        for v in [self.mark, self.figure, self.field_48, self.entered] {
            bytes.extend(v.to_le_bytes());
        }
        let end = bytes.iter().position(|&c| c == 0)?;
        bytes.truncate(end);
        Some(bytes)
    }

    /// The head's part of a new entry (`menu::bests::enter_tenth`): the key, group,
    /// nationality, name pointer, car, the taking part (if any), the player, the mark 1 and
    /// the name's first bytes. (A best time's seal and line are its tail's.)
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

/// A list of ten.
pub type List = [Entry; PER_LIST];

/// Entries `a` and `b`'s `fields` and names exchanged as the original does it (the name's
/// last byte stays); the name through `kept`, which ends with `a`'s.
pub fn exchange(
    list: &mut List,
    a: usize,
    b: usize,
    fields: &[Field],
    kept: &mut [u8; NAME_MOVED],
) {
    for &f in fields {
        let v = *list[a].field(f);
        *list[a].field(f) = *list[b].field(f);
        *list[b].field(f) = v;
    }
    kept.copy_from_slice(&list[a].name[..NAME_MOVED]);
    let from_b: [u8; NAME_MOVED] = list[b].name[..NAME_MOVED].try_into().expect("a name");
    list[a].name[..NAME_MOVED].copy_from_slice(&from_b);
    list[b].name[..NAME_MOVED].copy_from_slice(kept);
}

/// The tenth moved up: nine steps from the bottom, each pair exchanged when
/// `exchange_if(lower's key, upper's key)` (signed); the pair one up is next either way.
pub fn bubble_up(
    list: &mut List,
    fields: &[Field],
    exchange_if: fn(i32, i32) -> bool,
    kept: &mut [u8; NAME_MOVED],
) {
    for lower in (1..PER_LIST).rev() {
        let upper = lower - 1;
        if exchange_if(list[lower].key as i32, list[upper].key as i32) {
            exchange(list, upper, lower, fields, kept);
        }
    }
}

/// The fields an arcade level's score moves (the first eight of the arcade's best times').
pub const SCORE_FIELDS: [Field; 8] = [
    Field::NameId,
    Field::Key,
    Field::Player,
    Field::Nation,
    Field::Group,
    Field::Car,
    Field::RallyTime,
    Field::Mark,
];

/// An arcade level's ten best scores sorted (a lower score at least the upper's moves up).
pub fn sort_level_scores(list: &mut List, kept: &mut [u8; NAME_MOVED]) {
    bubble_up(list, &SCORE_FIELDS, |l, u| l >= u, kept);
}

/// No entry marked as just made.
pub fn clear_marks(list: &mut List) {
    for e in list {
        e.mark = 0;
    }
}

pub const LEVELS: usize = 7;
/// The levels' table's bytes in `var\hst.dat`.
pub const LEVELS_LEN: usize = LEVELS * PER_LIST * ENTRY_LEN;

/// The levels' best scores.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LevelScores {
    pub levels: [List; LEVELS],
}

impl crate::saves::tables::FileTable for LevelScores {
    const LEN: usize = LEVELS_LEN;

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

/// The made-up table (`saves::default_class_times`): `entries`
/// (`defaults::class_times`' seventy, level by level) over `t`: each its score, the
/// driver's +8 as the group, +0x14 the nationality, +0x18 the car, the name's string id
/// and the name's bytes.
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

    /// The typed sort against the byte one it replaces.
    #[test]
    fn sorting_matches_the_byte_version() {
        use crate::menu::standings::{Entries, sort_arcade_scores};
        let mut bytes: Vec<u8> = (0..PER_LIST * ENTRY_LEN)
            .map(|n| (n * 31 % 251) as u8)
            .collect();
        for n in 0..PER_LIST {
            put(
                &mut bytes,
                n * ENTRY_LEN + 0x14,
                [9, 9, 7, 7, 5, 3, 3, 2, 1, 8][n],
            );
        }
        let mut list: List = std::array::from_fn(|n| Entry::read(&bytes[n * ENTRY_LEN..]));
        let mut kept_bytes = [0; NAME_MOVED];
        sort_arcade_scores(
            &mut Entries {
                bytes: &mut bytes,
                size: ENTRY_LEN,
            },
            &mut kept_bytes,
        );
        let mut kept = [0; NAME_MOVED];
        sort_level_scores(&mut list, &mut kept);
        let mut typed = vec![0; PER_LIST * ENTRY_LEN];
        for (n, e) in list.iter().enumerate() {
            e.write(&mut typed[n * ENTRY_LEN..]);
        }
        assert_eq!(typed, bytes);
        assert_eq!(kept, kept_bytes);
    }

    /// Entering against the byte version.
    #[test]
    fn entering_matches_the_byte_version() {
        use crate::menu::bests::{clear_marks as clear_bytes, enter_tenth};
        use crate::menu::standings::Entries;
        let mut bytes: Vec<u8> = (0..PER_LIST * ENTRY_LEN).map(|n| (n * 7) as u8).collect();
        let mut list: List = std::array::from_fn(|n| Entry::read(&bytes[n * ENTRY_LEN..]));
        let best = NewBest {
            key: 5,
            player: 1,
            nationality: 2,
            name_at: 3,
            car: 4,
            group: 6,
            name: [b'q'; NAME_MOVED],
            entered: Some(8),
            seal: None,
            line: None,
        };
        let mut t = Entries {
            bytes: &mut bytes,
            size: ENTRY_LEN,
        };
        enter_tenth(&mut t, &best);
        list[PER_LIST - 1].enter(&best);
        let mut typed = vec![0; PER_LIST * ENTRY_LEN];
        for (n, e) in list.iter().enumerate() {
            e.write(&mut typed[n * ENTRY_LEN..]);
        }
        assert_eq!(typed, bytes);
        clear_bytes(&mut Entries {
            bytes: &mut bytes,
            size: ENTRY_LEN,
        });
        clear_marks(&mut list);
        for (n, e) in list.iter().enumerate() {
            e.write(&mut typed[n * ENTRY_LEN..]);
        }
        assert_eq!(typed, bytes);
    }
}
