//! The best scores (0xBB8 bytes in memory and in `var\hst.dat`): five groups (the classes; the
//! fifth the overall) of ten entries, best first.

pub const GROUPS: usize = 5;
pub(crate) const PER_GROUP: usize = 10;
pub const ENTRY_LEN: usize = 0x3C;
/// The table's bytes in `var\hst.dat`.
pub const TABLE_LEN: usize = GROUPS * PER_GROUP * ENTRY_LEN;
/// The name's room in bytes.
pub const NAME_ROOM: usize = 0x20;
/// What a ranking moves of a name (and what an entered name gets): its room less the last byte.
pub(crate) const NAME_MOVED: usize = 0x1F;

use crate::saves::name_id::NameId;

/// One best score (0x3C bytes).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScoreEntry {
    /// The player's name, written when entered and never moved by a ranking. `NameId::LIST_END`
    /// ends a list.
    pub name_id: NameId,
    /// The player (1 .. 4); 0 for a made-up entry, cleared after the pages.
    pub player: u32,
    /// The nationality.
    pub nation: u32,
    /// The car.
    pub car: u32,
    /// The group (the player's class).
    pub group: u32,
    /// 1 for an entry made in this event, cleared after the pages.
    pub new: u32,
    /// The name; a made-up entry's has no NUL, an entered one the player's first 0x1F
    /// bytes.
    pub name: [u8; NAME_ROOM],
    /// The score.
    pub score: u32,
}

fn dword(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(b[at..at + 4].try_into().expect("four bytes"))
}

fn put(b: &mut [u8], at: usize, v: u32) {
    b[at..at + 4].copy_from_slice(&v.to_le_bytes());
}

impl ScoreEntry {
    /// From its 0x3C bytes.
    pub fn read(b: &[u8; ENTRY_LEN]) -> Self {
        Self {
            name_id: NameId::from_file(dword(b, 0)),
            player: dword(b, 4),
            nation: dword(b, 8),
            car: dword(b, 0xC),
            group: dword(b, 0x10),
            new: dword(b, 0x14),
            name: b[0x18..0x38].try_into().expect("the name's room"),
            score: dword(b, 0x38),
        }
    }

    /// Into its 0x3C bytes.
    pub fn write(&self, b: &mut [u8; ENTRY_LEN]) {
        put(b, 0, self.name_id.to_file());
        put(b, 4, self.player);
        put(b, 8, self.nation);
        put(b, 0xC, self.car);
        put(b, 0x10, self.group);
        put(b, 0x14, self.new);
        b[0x18..0x38].copy_from_slice(&self.name);
        put(b, 0x38, self.score);
    }

    /// The name's text from +0x18 to the first NUL, which may lie past the name's room. None:
    /// no NUL before the entry's end.
    pub fn name_text(&self) -> Option<Vec<u8>> {
        let mut bytes = self.name.to_vec();
        bytes.extend(self.score.to_le_bytes());
        let end = bytes.iter().position(|&c| c == 0)?;
        bytes.truncate(end);
        Some(bytes)
    }
}

/// A group's ten.
pub type Group = [ScoreEntry; PER_GROUP];

/// The whole table.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BestScores {
    pub groups: [Group; GROUPS],
}

impl crate::saves::tables::FileTable for BestScores {
    fn zeroed(bytes: usize) -> Option<Self> {
        (bytes == TABLE_LEN).then(Self::default)
    }

    fn read(b: &[u8]) -> Self {
        let mut t = Self::default();
        for (n, e) in t.groups.as_flattened_mut().iter_mut().enumerate() {
            let at = n * ENTRY_LEN;
            *e = ScoreEntry::read(b[at..at + ENTRY_LEN].try_into().expect("an entry"));
        }
        t
    }

    fn write(&self) -> Vec<u8> {
        let mut b = vec![0; TABLE_LEN];
        for (n, e) in self.groups.as_flattened().iter().enumerate() {
            let at = n * ENTRY_LEN;
            e.write((&mut b[at..at + ENTRY_LEN]).try_into().expect("an entry"));
        }
        b
    }
}

/// Exchanges `upper` and `lower` but for `name_id` and the name's last byte; the name goes
/// through `kept`, which ends with the upper's.
fn exchange(ten: &mut Group, upper: usize, lower: usize, kept: &mut [u8; NAME_MOVED]) {
    let (a, b) = (ten[upper], ten[lower]);
    for (to, from) in [(upper, b), (lower, a)] {
        let e = &mut ten[to];
        e.score = from.score;
        e.new = from.new;
        e.player = from.player;
        e.nation = from.nation;
        e.group = from.group;
        e.car = from.car;
        e.name[..NAME_MOVED].copy_from_slice(&from.name[..NAME_MOVED]);
    }
    kept.copy_from_slice(&a.name[..NAME_MOVED]);
}

/// Moves a group's last entry up: nine steps from the bottom, each pair exchanged when the
/// lower score (signed) is at least the upper's.
pub(crate) fn rank_in_group(ten: &mut Group, kept: &mut [u8; NAME_MOVED]) {
    for lower in (1..PER_GROUP).rev() {
        let upper = lower - 1;
        if ten[lower].score as i32 >= ten[upper].score as i32 {
            exchange(ten, upper, lower, kept);
        }
    }
}

/// A best score to enter: player (1 .. 4), score, nationality, car, group, name id, name.
pub struct Best {
    pub player: u32,
    pub score: u32,
    pub nation: u32,
    pub car: u32,
    pub group: u32,
    pub name_id: NameId,
    pub name: [u8; NAME_MOVED],
}

/// A best score entered over the group's tenth (`new` 1), then moved up.
pub fn enter_best(ten: &mut Group, best: &Best, kept: &mut [u8; NAME_MOVED]) {
    let e = &mut ten[PER_GROUP - 1];
    e.score = best.score;
    e.player = best.player;
    e.new = 1;
    e.nation = best.nation;
    e.car = best.car;
    e.group = best.group;
    e.name_id = best.name_id;
    e.name[..NAME_MOVED].copy_from_slice(&best.name);
    rank_in_group(ten, kept);
}

/// After the pages: no entry new, none a player's.
pub fn clear_marks(t: &mut BestScores) {
    for e in t.groups.as_flattened_mut() {
        e.new = 0;
        e.player = 0;
    }
}

/// The made-up table (`saves::default_best_scores`): `entries` (fifty, group by group) over `t`:
/// each its score, the driver's +8 as the group, +0x14 the nationality, +0x18 the car and the
/// name's bytes.
pub fn fill_made_up(t: &mut BestScores, entries: &[(u32, crate::saves::defaults::Driver)]) {
    for (e, (score, d)) in t.groups.as_flattened_mut().iter_mut().zip(entries) {
        e.score = *score;
        e.group = d.fields[0];
        e.nation = d.fields[1];
        e.car = d.fields[2];
        e.name[..d.name.len()].copy_from_slice(&d.name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::saves::tables::FileTable;

    #[test]
    fn bytes_round_trip_at_the_originals_offsets() {
        let mut b = [0u8; TABLE_LEN];
        for (n, x) in b.iter_mut().enumerate() {
            *x = (n * 7) as u8;
        }
        let t = BestScores::read(&b);
        let e = t.groups[1][2];
        let at = 12 * ENTRY_LEN;
        assert_eq!(e.score, dword(&b, at + 0x38));
        assert_eq!(e.car, dword(&b, at + 0xC));
        assert_eq!(e.name[..], b[at + 0x18..at + 0x38]);
        assert_eq!(t.write(), b.to_vec());
    }

    #[test]
    fn ranking_moves_all_but_the_name_id_and_last_name_byte() {
        let mut ten = Group::default();
        for (n, e) in ten.iter_mut().enumerate() {
            e.score = 50 - n as u32;
            e.name_id = NameId::String(n as u32);
            e.name = [b'a' + n as u8; NAME_ROOM];
        }
        let mut kept = [0; NAME_MOVED];
        let best = Best {
            player: 2,
            score: 48,
            nation: 3,
            car: 4,
            group: 1,
            name_id: NameId::String(0x99),
            name: [b'z'; NAME_MOVED],
        };
        enter_best(&mut ten, &best, &mut kept);
        // Equal scores move up: the new 48 goes above the old one.
        assert_eq!((ten[2].score, ten[2].player, ten[2].name[0]), (48, 2, b'z'));
        assert_eq!(ten[2].name[NAME_MOVED], b'c', "the last byte stays");
        assert_eq!(ten[2].name_id, NameId::String(2), "the name id stays");
        assert_eq!((ten[3].score, ten[3].name[0], ten[3].new), (48, b'c', 0));
        assert_eq!(ten[9].name_id, NameId::String(0x99));
        assert_eq!(kept, [b'c'; NAME_MOVED]);
    }

    #[test]
    fn a_name_runs_on_into_the_score_without_its_nul() {
        let mut e = ScoreEntry {
            name: [b'x'; NAME_ROOM],
            score: 0x32,
            ..Default::default()
        };
        assert_eq!(e.name_text().unwrap().len(), NAME_ROOM + 1);
        e.score = u32::MAX;
        assert_eq!(e.name_text(), None);
    }
}
