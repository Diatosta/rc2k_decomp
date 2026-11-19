//! The championship's standings tables: entries of a fixed size (table A 0x9C, table B 0x78, the
//! best scores 0x3C) whose dwords are read by offset, as far as they are worked out. And the
//! tables' emptying, numbering and filling, the score marks' clearing and the computer team's
//! search.

/// A name's bytes in an entry.
pub const NAME_LEN: usize = 0x1F;
/// The entries the sorts look at.
pub const SORTED: usize = 0xB4;
/// "Any group" for the first-player searches.
pub const ANY_GROUP: u32 = 0x63;

/// A dword stored in a record: its byte offset and the value.
pub type Store = (usize, u32);

/// A table's entries, `size` bytes each.
pub struct Entries<'a> {
    pub bytes: &'a mut [u8],
    pub size: usize,
}

impl Entries<'_> {
    pub fn len(&self) -> usize {
        self.bytes.len() / self.size
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Entry `n`'s dword at `at`.
    pub fn get(&self, n: usize, at: usize) -> u32 {
        let o = n * self.size + at;
        u32::from_le_bytes(self.bytes[o..o + 4].try_into().unwrap())
    }

    pub fn set(&mut self, n: usize, at: usize, v: u32) {
        let o = n * self.size + at;
        self.bytes[o..o + 4].copy_from_slice(&v.to_le_bytes());
    }

    fn name(&self, n: usize, at: usize) -> [u8; NAME_LEN] {
        let o = n * self.size + at;
        self.bytes[o..o + NAME_LEN].try_into().unwrap()
    }

    pub fn set_name(&mut self, n: usize, at: usize, name: &[u8; NAME_LEN]) {
        self.set_bytes(n, at, name);
    }

    /// Entry `n`'s `len` bytes at `at`.
    pub fn bytes_at(&self, n: usize, at: usize, len: usize) -> &[u8] {
        let o = n * self.size + at;
        &self.bytes[o..o + len]
    }

    pub fn set_bytes(&mut self, n: usize, at: usize, bytes: &[u8]) {
        let o = n * self.size + at;
        self.bytes[o..o + bytes.len()].copy_from_slice(bytes);
    }

    /// Entry `n`'s stores made, in order.
    pub fn store(&mut self, n: usize, stores: &[Store]) {
        for &(at, v) in stores {
            self.set(n, at, v);
        }
    }

    /// Entries `a` and `b`'s dwords at `fields` and names at `name` swapped, the name through
    /// `kept` (which ends with `a`'s).
    pub fn swap(
        &mut self,
        a: usize,
        b: usize,
        fields: &[usize],
        name: usize,
        kept: &mut [u8; NAME_LEN],
    ) {
        for &o in fields {
            let v = self.get(a, o);
            self.set(a, o, self.get(b, o));
            self.set(b, o, v);
        }
        *kept = self.name(a, name);
        let nb = self.name(b, name);
        self.set_name(a, name, &nb);
        self.set_name(b, name, kept);
    }
}

/// Table A's fields a sort moves, and its name.
#[cfg(test)]
pub const A_FIELDS: [usize; 9] = [0xC, 0x14, 4, 0, 8, 0x1C, 0x10, 0x18, 0x4C];
#[cfg(test)]
pub const A_NAME: usize = 0x20;
/// Table B's name; its fields are `b_fields()`.
#[cfg(test)]
pub const B_NAME: usize = 0x14;

#[cfg(test)]
pub fn b_fields() -> Vec<usize> {
    let mut f = vec![0x34, 4, 0, 8, 0x10, 0xC];
    f.extend((0x38..=0x6C).step_by(4));
    f
}

/// The first `SORTED` entries sorted by the signed dword at `key` with an exchange sort:
/// for each i < j, entry j and i swapped (j first) when j's key is at most i's (`down` false) or at
/// least i's (`down`).
#[cfg(test)]
pub fn exchange_sort(
    t: &mut Entries,
    key: usize,
    down: bool,
    fields: &[usize],
    name: usize,
    kept: &mut [u8; NAME_LEN],
) {
    for i in 0..SORTED - 1 {
        for j in i + 1..SORTED {
            let (x, y) = (t.get(j, key) as i32, t.get(i, key) as i32);
            if if down { x >= y } else { x <= y } {
                t.swap(j, i, fields, name, kept);
            }
        }
    }
}

/// The arcade stage's points for a place: (the places less it, plus 1) x 1000.
pub fn arcade_points(places: u32, position: u32) -> u32 {
    places
        .wrapping_sub(position)
        .wrapping_add(1)
        .wrapping_mul(1000)
}

/// A list's entries after `shown` moved down one, up to and with the end mark at `end`.
pub fn remove_at(list: &mut [u32], shown: usize, end: usize) {
    list.copy_within(shown + 1..=end, shown);
}

/// What a first-player search finds: the entry, the computer drivers of the group before
/// it, its figure.
#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct First {
    pub entry: usize,
    pub computers: u32,
    pub figure: u32,
}

/// The search for the first player (`find1stplayer` in its fatal error): the entries of `group`
/// (`ANY_GROUP`: any whose group is not 0) whose player (+4) is not 0. None: a fatal error.
#[cfg(test)]
pub fn first_player(t: &Entries, group: u32, group_at: usize, figure_at: usize) -> Option<First> {
    let mut computers = 0;
    for n in 0..t.len() {
        let g = t.get(n, group_at);
        if !(if group == ANY_GROUP {
            g != 0
        } else {
            g == group
        }) {
            continue;
        }
        if t.get(n, 4) >= 1 {
            return Some(First {
                entry: n,
                computers,
                figure: t.get(n, figure_at),
            });
        }
        computers += 1;
    }
    None
}

/// What a search of a list ended with -1 found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Found {
    /// The entry's number.
    At(u32),
    /// The end mark met first.
    End,
    /// `limit` entries without the end mark.
    NoEnd,
}

/// The searches of the results pages (`menu::find_ranked_entry`, `find_ranked_position`,
/// `find_champ_position`, `first_overall_time`): from entry 0 on, the first `wanted`; entry 0 is
/// looked at before any end test, the others only if their first dword (`first(n)`) is not -1.
pub fn search(
    mut first: impl FnMut(u32) -> u32,
    mut wanted: impl FnMut(u32) -> bool,
    limit: u32,
) -> Found {
    for n in 0..limit {
        if wanted(n) {
            return Found::At(n);
        }
        if first(n + 1) == u32::MAX {
            return Found::End;
        }
    }
    Found::NoEnd
}

/// The entries the tables have room for.
pub const ROOM: u32 = 200;
/// A time beyond any (an emptied table A entry's two times).
pub const NO_TIME: u32 = 0x36_F272;
/// Table A's emptied entry: its first eight dwords.
#[cfg(test)]
pub const EMPTY_A: [u32; 8] = [u32::MAX, u32::MAX, 0, 0, 0, NO_TIME, NO_TIME, 0];
/// `EMPTY_A` stored in an entry's bytes (the typed version's test).
#[cfg(test)]
pub fn empty_bytes_a(b: &mut [u8]) {
    for (k, v) in EMPTY_A.into_iter().enumerate() {
        b[4 * k..4 * k + 4].copy_from_slice(&v.to_le_bytes());
    }
}

/// Table B's emptied entry: its first five dwords.
#[cfg(test)]
pub const EMPTY_B: [u32; 5] = [u32::MAX; 5];

/// Which entries a numbering counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Numbering {
    /// Those of this group.
    Group(u32),
    /// All, or with `grouped` only those with a group (not 0).
    All { grouped: bool },
}

impl Numbering {
    pub fn counts(self, group: u32) -> bool {
        match self {
            Numbering::Group(g) => group == g,
            Numbering::All { grouped } => !grouped || group != 0,
        }
    }
}

/// The numbers 1, 2, .. the counted entries get in order (by their groups); `None` for the
/// others.
pub fn number(groups: &[u32], which: Numbering) -> Vec<Option<u32>> {
    let mut n = 0;
    groups
        .iter()
        .map(|&g| {
            which.counts(g).then(|| {
                n += 1;
                n
            })
        })
        .collect()
}

/// Table A's figure and table B's, each entry's emptied (0) by `menu::clear_standings_a` /
/// `_b`.
#[cfg(test)]
pub const A_FIGURE: usize = 0x44;
#[cfg(test)]
pub const B_FIGURE: usize = 0x70;

/// What `menu::copy_standing_entry` takes from a player's record for a table A entry.
pub struct PlayerA {
    /// The player's index (0 .. 3).
    pub index: u32,
    /// The stage time (+0x88), the group (+0x90), the total (+0x8C).
    pub time: u32,
    pub group: u32,
    pub total: u32,
    /// The name's pointer (+0x18) and its first 0x20 bytes.
    pub name_at: u32,
    pub name: [u8; 0x20],
    /// The nationality (+4), the car's number (+8), the taking part (+0xD0).
    pub nationality: u32,
    pub car: u32,
    pub entered: u32,
}

/// A table A entry filled from a player: +4 the index + 1, +0x14 the time, +0x1C the
/// group, +0x18 the total, +0xC the name's pointer and +0x20 the name, +8 the nationality,
/// +0x10 the car, +0x4C the taking part.
#[cfg(test)]
pub fn fill_a(t: &mut Entries, n: usize, p: &PlayerA) {
    t.store(
        n,
        &[
            (4, p.index.wrapping_add(1)),
            (0x14, p.time),
            (0x1C, p.group),
            (0x18, p.total),
            (0xC, p.name_at),
        ],
    );
    t.set_bytes(n, 0x20, &p.name);
    t.store(n, &[(8, p.nationality), (0x10, p.car), (0x4C, p.entered)]);
}

/// What `menu::player_to_standing_b` takes from a player's record for a table B entry.
pub struct PlayerB {
    /// The player's index (0 .. 3).
    pub index: u32,
    /// The name's first 0x20 bytes.
    pub name: [u8; 0x20],
    /// The nationality (+4), the car's number (+8), the group (+0x90), the total (+0x8C).
    pub nationality: u32,
    pub car: u32,
    pub group: u32,
    pub total: u32,
    /// The seven dwords at +0xB4 and the seven at +0x98.
    pub b4: [u32; 7],
    pub slots: [u32; 7],
}

/// A table B entry filled from a player: +4 the index + 1, +0x14 the name, +8 / +0xC /
/// +0x10 the nationality, car and group, +0x34 the total, +0x38 .. +0x50 its +0xB4 ..,
/// +0x54 .. +0x6C its +0x98 ...
#[cfg(test)]
pub fn fill_b(t: &mut Entries, n: usize, p: &PlayerB) {
    t.set(n, 4, p.index.wrapping_add(1));
    t.set_bytes(n, 0x14, &p.name);
    t.store(
        n,
        &[
            (8, p.nationality),
            (0xC, p.car),
            (0x10, p.group),
            (0x34, p.total),
        ],
    );
    for k in 0..7 {
        t.set(n, 0x38 + 4 * k, p.b4[k]);
        t.set(n, 0x54 + 4 * k, p.slots[k]);
    }
}

/// Table A's entry `n` filled from driver `d` of `drivers` (`menu::standings_from_drivers`):
/// +0x14 / +0x1C / +0x18 / +8 / +0x10 / +0 / +0x4C its +0xC / +8 / +0x10 / +0x14 / +0x18 /
/// +0 / +0x24, +4 0, +0xC its name's string number and +0x20 the text (with its NUL).
#[cfg(test)]
pub fn fill_a_from_driver(
    t: &mut Entries,
    n: usize,
    drivers: &Entries,
    d: usize,
    name_id: u32,
    text: &[u8],
) {
    let f = |at| drivers.get(d, at);
    t.store(
        n,
        &[
            (0x14, f(0xC)),
            (4, 0),
            (0x1C, f(8)),
            (0x18, f(0x10)),
            (8, f(0x14)),
            (0x10, f(0x18)),
            (0, f(0)),
            (0x4C, f(0x24)),
            (0xC, name_id),
        ],
    );
    t.set_bytes(n, 0x20, text);
}

/// Table B's entry `n` filled from driver `d` (`menu::standings_b_from_all`): +4 0, +0x10 /
/// +8 / +0xC / +0 / +0x34 its +8 / +0x14 / +0x18 / +0 / +0x10, +0x38 .. +0x6C its +0x5C ..
/// +0x90, +0x14 the text of its name (with its NUL).
#[cfg(test)]
pub fn fill_b_from_driver(t: &mut Entries, n: usize, drivers: &Entries, d: usize, text: &[u8]) {
    let f = |at| drivers.get(d, at);
    t.store(
        n,
        &[
            (4, 0),
            (0x10, f(8)),
            (8, f(0x14)),
            (0xC, f(0x18)),
            (0, f(0)),
            (0x34, f(0x10)),
        ],
    );
    for k in 0..14 {
        t.set(n, 0x38 + 4 * k, f(0x5C + 4 * k));
    }
    t.set_bytes(n, 0x14, text);
}

/// Where the players go in a table filled from all (`menu::standings_a_from_all`,
/// `standings_b_from_all`): each group's (0 .. 3) players, in order, into the entries from the
/// group's first (`firsts`) down. (player, entry) pairs.
pub fn by_group(groups: &[u32], firsts: [usize; 4]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for (group, first) in firsts.into_iter().enumerate() {
        let mut entry = first;
        for (n, &g) in groups.iter().enumerate() {
            if g == group as u32 {
                out.push((n, entry));
                entry = entry.wrapping_sub(1);
            }
        }
    }
    out
}

/// The entries filled from all: table A's from 89, 139, 169 and 179; table B's from 89,
/// 119, 169 and 179.
pub const A_GROUP_FIRSTS: [usize; 4] = [89, 139, 169, 179];
pub const B_GROUP_FIRSTS: [usize; 4] = [89, 119, 169, 179];

/// The summary's rows (`menu::copy_standings_summary`).
#[cfg(test)]
pub const SUMMARY_ROWS: usize = 0x5A;

/// The summary's rows (+0, +4, +8, +0xC of each: the entry's +0xC, +8, +0x10, +0x14) from table A's
/// first `SUMMARY_ROWS` entries with a group (+0x1C not 0; `grouped`) or without. None when the
/// table runs out first.
#[cfg(test)]
pub fn summary_rows(t: &Entries, grouped: bool) -> Option<Vec<[u32; 4]>> {
    let rows: Vec<[u32; 4]> = (0..t.len())
        .filter(|&n| (t.get(n, 0x1C) != 0) == grouped)
        .take(SUMMARY_ROWS)
        .map(|n| [t.get(n, 0xC), t.get(n, 8), t.get(n, 0x10), t.get(n, 0x14)])
        .collect();
    (rows.len() == SUMMARY_ROWS).then_some(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standings() {
        let mut bytes = vec![0u8; 4 * 0x20];
        let mut t = Entries {
            bytes: &mut bytes,
            size: 0x20,
        };
        t.set(0, 0x10, 2);
        t.set(1, 0x10, 1);
        t.set(2, 0x10, 1);
        t.set(2, 4, 3);
        t.set(2, 8, 77);
        assert_eq!(
            first_player(&t, 1, 0x10, 8),
            Some(First {
                entry: 2,
                computers: 1,
                figure: 77
            })
        );
        assert_eq!(first_player(&t, 2, 0x10, 8), None);
        let mut l = [1, 2, 3, u32::MAX, 9];
        remove_at(&mut l, 1, 3);
        assert_eq!(l, [1, 3, u32::MAX, u32::MAX, 9]);
        assert_eq!(arcade_points(6, 2), 5000);
    }

    #[test]
    fn numbering() {
        let groups = [1, 0, 2, 1, 0];
        assert_eq!(
            number(&groups, Numbering::Group(1)),
            [Some(1), None, None, Some(2), None]
        );
        assert_eq!(
            number(&groups, Numbering::All { grouped: true }),
            [Some(1), None, Some(2), Some(3), None]
        );
        assert_eq!(
            number(&groups, Numbering::All { grouped: false })[4],
            Some(5)
        );
        assert_eq!(number(&groups, Numbering::Group(7)), [None; 5]);
    }

    #[test]
    fn searches() {
        let list = [(0, 5), (0, 6), (0, 7), (u32::MAX, 8)];
        let first = |n: u32| list[n as usize].0;
        assert_eq!(search(first, |n| list[n as usize].1 == 5, 9), Found::At(0));
        assert_eq!(search(first, |n| list[n as usize].1 == 7, 9), Found::At(2));
        assert_eq!(search(first, |n| list[n as usize].1 == 8, 9), Found::End);
        assert_eq!(search(|_| 0, |_| false, 3), Found::NoEnd);
    }

    #[test]
    fn fills() {
        let mut bytes = vec![0u8; 3 * 0x9C];
        let mut t = Entries {
            bytes: &mut bytes,
            size: 0x9C,
        };
        let p = PlayerA {
            index: 2,
            time: 10,
            group: 1,
            total: 20,
            name_at: 0x1234,
            name: [b'n'; 0x20],
            nationality: 5,
            car: 6,
            entered: 1,
        };
        fill_a(&mut t, 1, &p);
        assert_eq!((t.get(1, 4), t.get(1, 0x18), t.get(1, 0x4C)), (3, 20, 1));
        assert_eq!(t.bytes_at(1, 0x20, 0x20), &[b'n'; 0x20]);
        assert_eq!(t.get(1, 0x40), 0);
        let mut drivers = vec![0u8; 2 * 0x94];
        let mut d = Entries {
            bytes: &mut drivers,
            size: 0x94,
        };
        d.set(1, 0, 7);
        d.set(1, 0x24, 9);
        d.set(1, 0x5C + 4 * 13, 11);
        fill_a_from_driver(&mut t, 2, &d, 1, 0x55, b"ab\0");
        assert_eq!((t.get(2, 0), t.get(2, 0x4C), t.get(2, 0xC)), (7, 9, 0x55));
        assert_eq!(t.bytes_at(2, 0x20, 3), b"ab\0");
        let mut bytes = vec![0u8; 0x78];
        let mut b = Entries {
            bytes: &mut bytes,
            size: 0x78,
        };
        fill_b_from_driver(&mut b, 0, &d, 1, b"x\0");
        assert_eq!((b.get(0, 0), b.get(0, 0x6C)), (7, 11));
        let q = PlayerB {
            index: 0,
            name: [b'q'; 0x20],
            nationality: 1,
            car: 2,
            group: 3,
            total: 4,
            b4: [5; 7],
            slots: [6; 7],
        };
        fill_b(&mut b, 0, &q);
        assert_eq!(
            (b.get(0, 4), b.get(0, 0x34), b.get(0, 0x50), b.get(0, 0x6C)),
            (1, 4, 5, 6)
        );
        assert_eq!(b.bytes_at(0, 0x14, 0x20), &[b'q'; 0x20]);
    }

    #[test]
    fn groups_and_summary() {
        assert_eq!(
            by_group(&[1, 0, 1, 3], A_GROUP_FIRSTS),
            [(1, 89), (0, 139), (2, 138), (3, 179)]
        );
        let mut bytes = vec![0u8; 200 * 0x9C];
        let mut t = Entries {
            bytes: &mut bytes,
            size: 0x9C,
        };
        for n in 0..200 {
            t.set(n, 0x1C, (n % 2) as u32);
            t.set(n, 0x14, n as u32);
        }
        let rows = summary_rows(&t, true).unwrap();
        assert_eq!((rows[0][3], rows[89][3]), (1, 179));
        assert!(
            summary_rows(
                &Entries {
                    bytes: &mut [0u8; 0x9C * 3],
                    size: 0x9C
                },
                false
            )
            .is_none()
        );
    }
}
