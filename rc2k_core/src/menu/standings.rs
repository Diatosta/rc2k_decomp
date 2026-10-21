//! The championship's standings tables (converted 2026-10-21 from `menu::sort_standings_by_*`,
//! `menu::sort_standings_b_by_*`, `menu::rank_best_score*`, `menu::enter_best_score*` and the
//! three first-player searches, and the arcade's `menu::sort_arcade_stage_times` 0x48A0CE,
//! `sort_arcade_level_scores` 0x48A2AE, `score_arcade_stage` 0x48A383, `remove_shown_car`
//! 0x486BF5): entries of a fixed size (table A 0x9C, table B 0x78, the
//! best scores 0x3C) whose dwords are read by offset, as far as they are worked out. And the
//! tables' emptying and numbering (`menu::init_standings_a` 0x4730D8 / `_b` 0x4A4644,
//! `number_standings_a` 0x4A2055 / `_b` 0x4A4A9D, `number_all_a` 0x4A2090 / `_b` 0x4A5A98,
//! `clear_standings_a` 0x4A5A14 / `_b` 0x4A5A2F).

/// A name's bytes in an entry.
pub const NAME_LEN: usize = 0x1F;
/// The entries the sorts look at.
pub const SORTED: usize = 0xB4;
/// "Any group" for the first-player searches.
pub const ANY_GROUP: u32 = 0x63;

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

    fn set_name(&mut self, n: usize, at: usize, name: &[u8; NAME_LEN]) {
        let o = n * self.size + at;
        self.bytes[o..o + NAME_LEN].copy_from_slice(name);
    }

    /// Entries `a` and `b`'s dwords at `fields` and names at `name` swapped, the name
    /// through `kept` (which ends with `a`'s), as the original.
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
pub const A_FIELDS: [usize; 9] = [0xC, 0x14, 4, 0, 8, 0x1C, 0x10, 0x18, 0x4C];
pub const A_NAME: usize = 0x20;
/// Table B's name; its fields are `b_fields()`.
pub const B_NAME: usize = 0x14;

pub fn b_fields() -> Vec<usize> {
    let mut f = vec![0x34, 4, 0, 8, 0x10, 0xC];
    f.extend((0x38..=0x6C).step_by(4));
    f
}

/// The first `SORTED` entries sorted by the signed dword at `key` with the original's
/// exchange sort: for each i < j, entry j and i swapped (j first) when j's key is at most
/// i's (`down` false) or at least i's (`down`).
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

/// The best score's fields a rank moves, and its name.
pub const BEST_FIELDS: [usize; 6] = [0x38, 0x14, 4, 8, 0x10, 0xC];
pub const BEST_NAME: usize = 0x18;

/// A group's ten best scores: the last moved up (nine compares from the bottom: a lower
/// entry not below the one above it moves up).
pub fn rank_in_group(ten: &mut Entries, kept: &mut [u8; NAME_LEN]) {
    bubble_up(
        ten,
        0x38,
        &BEST_FIELDS,
        BEST_NAME,
        |lower, upper| lower >= upper,
        kept,
    );
}

/// Ten entries' last moved up: nine steps from the bottom, each pair (upper, lower)
/// exchanged when `exchange(lower's key, upper's key)` (signed); the pair one up is next
/// either way.
pub fn bubble_up(
    ten: &mut Entries,
    key: usize,
    fields: &[usize],
    name: usize,
    exchange: fn(i32, i32) -> bool,
    kept: &mut [u8; NAME_LEN],
) {
    for lower in (1..10).rev() {
        let upper = lower - 1;
        if exchange(ten.get(lower, key) as i32, ten.get(upper, key) as i32) {
            ten.swap(upper, lower, fields, name, kept);
        }
    }
}

/// The arcade's best times' fields an exchange moves (their key +0x14), and the scores'
/// (the first eight); the names at +0x20.
pub const ARCADE_TIME_FIELDS: [usize; 27] = [
    0xC, 0x14, 4, 8, 0x1C, 0x10, 0x18, 0x40, 0x50, 0x54, 0x58, 0x5C, 0x60, 0x64, 0x68, 0x6C, 0x70,
    0x74, 0x78, 0x7C, 0x80, 0x84, 0x88, 0x8C, 0x90, 0x94, 0x98,
];
pub const ARCADE_NAME: usize = 0x20;

/// The arcade course's ten best times sorted (a lower time at most the upper's moves up).
pub fn sort_arcade_times(ten: &mut Entries, kept: &mut [u8; NAME_LEN]) {
    bubble_up(
        ten,
        0x14,
        &ARCADE_TIME_FIELDS,
        ARCADE_NAME,
        |l, u| l <= u,
        kept,
    );
}

/// The arcade level's ten best scores sorted (a lower score at least the upper's moves up).
pub fn sort_arcade_scores(ten: &mut Entries, kept: &mut [u8; NAME_LEN]) {
    bubble_up(
        ten,
        0x14,
        &ARCADE_TIME_FIELDS[..8],
        ARCADE_NAME,
        |l, u| l >= u,
        kept,
    );
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

/// A best score to enter: the player's number (1 .. 4), score, +4 and +8, group (+0x90),
/// name pointer and name.
pub struct Best {
    pub player: u32,
    pub score: u32,
    pub car: u32,
    pub field_8: u32,
    pub group: u32,
    pub name_at: u32,
    pub name: [u8; NAME_LEN],
}

/// A best score entered at the bottom of its group's ten (entry 9: +0x38 the score, +4 the
/// player, +0x14 1, +8 / +0xC / +0x10, +0 the name pointer and +0x18 the name), then moved
/// up.
pub fn enter_best(ten: &mut Entries, best: &Best, kept: &mut [u8; NAME_LEN]) {
    ten.set(9, 0x38, best.score);
    ten.set(9, 4, best.player);
    ten.set(9, 0x14, 1);
    ten.set(9, 8, best.car);
    ten.set(9, 0xC, best.field_8);
    ten.set(9, 0x10, best.group);
    ten.set(9, 0, best.name_at);
    ten.set_name(9, BEST_NAME, &best.name);
    rank_in_group(ten, kept);
}

/// What a first-player search finds: the entry, the computer drivers of the group before
/// it, its figure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct First {
    pub entry: usize,
    pub computers: u32,
    pub figure: u32,
}

/// The original's search for the first player (`find1stplayer`): the entries of `group`
/// (`ANY_GROUP`: any whose group is not 0) whose player (+4) is not 0. None: the game's
/// fatal error.
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

/// The searches of the results pages (`menu::find_ranked_entry` 0x4A2547,
/// `find_ranked_position` 0x4A2584, `find_champ_position` 0x4A5A85,
/// `first_overall_time` 0x4A327A): from entry 0 on, the first `wanted`; entry 0 is looked at
/// before any end test, the others only if their first dword (`first(n)`) is not -1.
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
pub const EMPTY_A: [u32; 8] = [u32::MAX, u32::MAX, 0, 0, 0, NO_TIME, NO_TIME, 0];
/// Table B's emptied entry: its first five dwords.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standings() {
        let mut bytes = vec![0u8; 10 * 0x3C];
        let mut t = Entries {
            bytes: &mut bytes,
            size: 0x3C,
        };
        for n in 0..10 {
            t.set(n, 0x38, 50 - n as u32);
            t.bytes[n * 0x3C + BEST_NAME] = b'a' + n as u8;
        }
        t.set(9, 0x38, 48);
        let mut kept = [0; NAME_LEN];
        rank_in_group(&mut t, &mut kept);
        assert_eq!((t.get(2, 0x38), t.bytes[2 * 0x3C + BEST_NAME]), (48, b'j'));
        assert_eq!((t.get(3, 0x38), t.bytes[3 * 0x3C + BEST_NAME]), (48, b'c'));
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
}
