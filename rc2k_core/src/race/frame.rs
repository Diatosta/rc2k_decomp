//! The race's frame: the panel's fields, the course grid round the view, the sort
//! entries and the objects' bytes (converted 2026-10-21 from `race::split_colours`
//! 0x45B16A, `both_split_colours` 0x45B147, `panel_players` 0x45ACDB, `panel_neighbours`
//! 0x45AD36, `panel_marks` 0x45AEFE, `hold_to_limit` 0x4650A9, `lamps_wanted` 0x4536D7,
//! `add_sort_entry` 0x454D57, `course_grid` 0x45497E, `checkpoint_colours` 0x4548E3 and
//! `object_bytes` 0x455263).

use crate::Invalid;

/// The dwords each view's colour buffer holds (`dword_6D160C`, `dword_6D170C`).
pub const COLOUR_BUFFER_DWORDS: u32 = 0x40;

/// The colour of a stretch not reached (`split_colours`' fill).
pub const SPLIT_UNREACHED: u32 = 0xC;
/// The colour of a split not above the one before.
pub const SPLIT_SLOWER: u32 = 8;
/// The colour of a split above the one before.
pub const SPLIT_FASTER: u32 = 4;

/// `race::split_colours`' refusal: a fill or a count of splits (when positive) past the
/// colour buffer.
pub fn check_split_colours(fill: u32, count: u32) -> Result<(), Invalid> {
    if fill > COLOUR_BUFFER_DWORDS || (count as i32 > 0 && count > COLOUR_BUFFER_DWORDS) {
        return Err(Invalid("a count past the colour buffer's 0x40 dwords"));
    }
    Ok(())
}

/// `race::split_colours`: each split's colour: `SPLIT_SLOWER` when it is not above the one
/// before (0x63 before the first; the sign of the difference, as `cmp` / `jns`), else
/// `SPLIT_FASTER`.
pub fn split_colours(splits: &[u32]) -> Vec<u32> {
    let mut before = 0x63u32;
    splits
        .iter()
        .map(|&s| {
            let colour = if (before.wrapping_sub(s) as i32) < 0 {
                SPLIT_FASTER
            } else {
                SPLIT_SLOWER
            };
            before = s;
            colour
        })
        .collect()
}

/// What `race::panel_players` does to the panel's player fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PanelPlayers {
    /// Nothing (a race mode set, `dword_62389C`).
    Kept,
    /// The two name fields unset (-1): one player.
    One,
    /// All four unset, then the two name fields the view blocks' +0xFA4.
    Two,
}

/// `race::panel_players`: by the race's mode and (asked only in mode 0) the players.
pub fn panel_players(mode: u32, players: impl FnOnce() -> u32) -> PanelPlayers {
    if mode != 0 {
        PanelPlayers::Kept
    } else if players() == 1 {
        PanelPlayers::One
    } else {
        PanelPlayers::Two
    }
}

/// `race::panel_neighbours`: a neighbour's car number on the panel: -1 when there is
/// none (the view's car at the end it would be past), else its number.
pub fn neighbour_number(at_end: bool, number: u32) -> u32 {
    if at_end { u32::MAX } else { number }
}

/// The marks one view shows on the panel (`race::panel_marks`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ViewMarks {
    /// 400 + the block's +0xFA0.
    pub base: u32,
    /// The marks shown (the block's +0xF98, at most 4; 0 when none).
    pub shown: u32,
    /// The first `shown` marks (+0xF78 ..), each + 1; the rest 0.
    pub marks: [u32; 4],
}

/// The most marks a view shows.
pub const MOST_MARKS: u32 = 4;

/// `race::panel_marks` for one view: from its +0xFA0, its count (+0xF98) and its marks
/// (asked for the first `shown` only).
pub fn view_marks(offset: u32, count: u32, mark: impl Fn(u32) -> u32) -> ViewMarks {
    let shown = count.min(MOST_MARKS);
    let mut marks = [0u32; 4];
    for (k, m) in marks.iter_mut().enumerate().take(shown as usize) {
        *m = mark(k as u32).wrapping_add(1);
    }
    ViewMarks {
        base: 0x190u32.wrapping_add(offset),
        shown,
        marks,
    }
}

/// `race::hold_to_limit`: `value` held to `limit` (`dword_71C6D4`, unsigned).
pub fn hold_to_limit(value: u32, limit: u32) -> u32 {
    value.min(limit)
}

/// `race::lamps_wanted`: the lamp count with bit 0 set when a car's lights are wanted.
pub fn lamps_wanted(lamps: u32, any_lit: bool) -> u32 {
    if any_lit { lamps | 1 } else { lamps }
}

/// `race::lamps_wanted`: whether a race car's lights are wanted: its parts shown
/// (+0x25C) and its lights on (+0x84, asked only with its parts shown).
pub fn car_lit(parts_shown: u32, lights: impl FnOnce() -> u32) -> bool {
    parts_shown != 0 && lights() != 0
}

/// The keys past which `add_sort_entry` leaves the first dword 0.
pub const SORT_KEY_LIMIT: u32 = 0x100_0000;

/// One sort entry (`race::add_sort_entry`): its first dword and the key (ebx & 0x7FFFFFFF).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SortEntry {
    /// None: left 0 (a key of 0x1000000 or more).
    pub first: Option<u32>,
    pub key: u32,
}

/// `race::add_sort_entry`: the entry for `what` (its second dword) at `depth` (eax):
/// (depth << 16) | the key's low word, for a key below 0x1000000.
pub fn sort_entry(depth: u32, what: u32) -> SortEntry {
    let key = what & 0x7FFF_FFFF;
    SortEntry {
        first: (key < SORT_KEY_LIMIT).then_some((depth << 16) | (key & 0xFFFF)),
        key,
    }
}

/// The course grid's cells a side (`race::course_grid`).
pub const GRID_SIDE: u32 = 32;
/// An empty cell.
pub const GRID_EMPTY: u32 = 0x0FFF_FFFF;

/// `race::course_grid`: the grid's corner: the view's place (`dword_6062B0` /
/// `dword_6062B4`, >> 21 arithmetic) less the half size.
pub fn grid_corner(view_x: u32, view_z: u32, half: u32) -> (u32, u32) {
    (
        (((view_x as i32) >> 21) as u32).wrapping_sub(half),
        (((view_z as i32) >> 21) as u32).wrapping_sub(half),
    )
}

/// A course record's place (+0: two 11-bit cell numbers, -1 none).
pub fn record_cell(place: u32) -> Option<(u32, u32)> {
    (place != u32::MAX).then_some((place & 0x7FF, place >> 11 & 0x7FF))
}

/// `race::course_grid`: the cell a record at (`x`, `z`) goes in, row z column x from the
/// corner (& 0x7FF), when within the span on both axes.
pub fn grid_cell(x: u32, z: u32, corner: (u32, u32), span: u32) -> Option<u32> {
    let column = x.wrapping_sub(corner.0) & 0x7FF;
    if column > span {
        return None;
    }
    let row = z.wrapping_sub(corner.1) & 0x7FF;
    if row > span {
        return None;
    }
    Some(row.wrapping_mul(GRID_SIDE).wrapping_add(column))
}

/// The colour levels `race::checkpoint_colours` works out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CheckpointColours {
    /// The face's +0x10 low nibble (0 read as 0x10): `dword_614D08`.
    pub near: u32,
    /// Its high nibble: `dword_614D0C`.
    pub far: u32,
    /// The checkpoint's distance less the car's: `dword_614D10`.
    pub distance: u32,
    /// The level used (`dword_63F17C`): near when twice the distance (its size) is below
    /// 0x40000000 (signed), far from 0xC0000000, else the larger.
    pub level: u32,
}

/// `race::checkpoint_colours`: from the face's colour byte, the checkpoint's distance
/// along the line and the car's.
pub fn checkpoint_colours(byte: u8, checkpoint: u32, car: u32) -> CheckpointColours {
    let nibble = |v: u32| if v == 0 { 0x10 } else { v };
    let near = nibble(byte as u32 & 0xF);
    let far = nibble(byte as u32 >> 4);
    let distance = checkpoint.wrapping_sub(car);
    let e = (distance as i32).wrapping_abs() as u32;
    let e = e.wrapping_add(e);
    let level = if (e.wrapping_sub(0x4000_0000) as i32) < 0 {
        near
    } else if (e.wrapping_sub(0xC000_0000) as i32) >= 0 {
        far
    } else {
        near.max(far)
    };
    CheckpointColours {
        near,
        far,
        distance,
        level,
    }
}

/// `race::object_bytes`: an object's four bytes: its record's +0xE bit 0, 1, its kind's
/// byte (`kind`), and its first face's kind's byte (`face_kind`, with faces) or 0xFF.
pub fn object_bytes(flags: u8, kind: u32, face_kind: Option<u32>) -> [u8; 4] {
    [
        flags & 1,
        1,
        kind as u8,
        face_kind.map_or(0xFF, |k| k as u8),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits() {
        assert!(check_split_colours(0x41, 0).is_err());
        assert!(check_split_colours(0x40, 0x41).is_err());
        assert!(check_split_colours(0x40, 0x8000_0000).is_ok());
        assert_eq!(split_colours(&[0x63, 0x64, 3, 3]), [8, 4, 8, 8]);
        assert_eq!(split_colours(&[0x8000_0063]), [4]);
    }

    #[test]
    fn panel() {
        assert_eq!(panel_players(1, || unreachable!()), PanelPlayers::Kept);
        assert_eq!(panel_players(0, || 1), PanelPlayers::One);
        assert_eq!(panel_players(0, || 2), PanelPlayers::Two);
        assert_eq!(neighbour_number(true, 5), u32::MAX);
        let m = view_marks(10, 6, |k| k * 10);
        assert_eq!((m.base, m.shown, m.marks), (410, 4, [1, 11, 21, 31]));
        assert_eq!(view_marks(0, 0, |_| unreachable!()).shown, 0);
        assert_eq!(hold_to_limit(11, 10), 10);
        assert_eq!(lamps_wanted(6, true), 7);
        assert!(car_lit(1, || 2) && !car_lit(0, || unreachable!()));
    }

    #[test]
    fn sorts_and_grid() {
        assert_eq!(
            sort_entry(3, 0x8000_1234),
            SortEntry {
                first: Some(0x3_1234),
                key: 0x1234
            }
        );
        assert_eq!(sort_entry(3, 0x0100_0000).first, None);
        assert_eq!(
            grid_corner(0x0060_0000, 0xFFE0_0000, 2),
            (1, (-3i32) as u32)
        );
        assert_eq!(record_cell(u32::MAX), None);
        assert_eq!(record_cell(5 | 7 << 11), Some((5, 7)));
        assert_eq!(grid_cell(5, 7, (4, 4), 4), Some(3 * 32 + 1));
        assert_eq!(grid_cell(9, 7, (4, 4), 4), None);
        assert_eq!(grid_cell(3, 7, (4, 4), 4), None);
    }

    #[test]
    fn colours_and_objects() {
        let c = checkpoint_colours(0x30, 10, 4);
        assert_eq!((c.near, c.far, c.distance, c.level), (0x10, 3, 6, 0x10));
        assert_eq!(checkpoint_colours(0x21, 0x3000_0000, 0).level, 2);
        // The sign tests leave the far level unreachable.
        assert_eq!(checkpoint_colours(0x21, 0x7000_0000, 0).level, 1);
        assert_eq!(checkpoint_colours(0x21, 0x2000_0000, 0).level, 2);
        assert_eq!(object_bytes(3, 0x1FF, None), [1, 1, 0xFF, 0xFF]);
        assert_eq!(object_bytes(2, 7, Some(9)), [0, 1, 7, 9]);
    }
}
