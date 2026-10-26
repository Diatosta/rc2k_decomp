//! The list menus (the replay viewer's, `sub_4144CF`'s): their items' marks and kinds, the
//! highlight's moves and the keys' repeat count (converted 2026-10-21 from
//! `menu::list_clear_marks` 0x414AC9, `menu::list_item_unusable` 0x414C1B,
//! `menu::list_step` 0x414A55 and `menu::list_take_key` 0x41469A). An item is 16 bytes:
//! its first byte the marks (bits 6 and 7) and the kind (low nibble); a kind 2 item has
//! choices, 2 bytes each, whose first bytes have marks too.

/// An item's or a choice's first byte without its marks.
pub fn unmarked(byte: u8) -> u8 {
    byte & 0x3F
}

/// Whether an item of this first byte has choices (kind 2).
pub fn has_choices(byte: u8) -> bool {
    byte & 0xF == 2
}

/// The rounds a count byte's loop makes: 0 goes round 256 times, as the original does.
pub fn rounds(count: u8) -> u32 {
    if count == 0 { 0x100 } else { count as u32 }
}

/// `menu::list_item_unusable`: an item of this first byte cannot be chosen: marked exactly
/// 0x40 or 0x80, or of kind 5.
pub fn unusable(byte: u8) -> bool {
    matches!(byte & 0xC0, 0x40 | 0x80) || byte & 0xF == 5
}

/// The keys that move the highlight, in the order of a neighbours table's entries: back
/// and on, and the other way.
pub const MOVES: [u8; 4] = [0x66, 0x6B, 0x68, 0x69];

/// What a key does to the highlight.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// Moved to this item.
    Moved(u8),
    /// The table's entry is negative (this): not moved.
    Blocked(u8),
    /// 0x68 or 0x69 without a table: not moved.
    NoTable,
    /// Not a moving key; whether it is below 0x69.
    Other { below: bool },
}

/// `menu::list_step`: `key` on the highlight `from` (its low byte) of a list of `count`
/// items, `table` whether there is a neighbours table and `entry` its entry for the key's
/// place (0 .. 3), asked only when there is one.
pub fn step(key: u8, from: u8, count: u8, table: bool, entry: impl Fn(usize) -> u8) -> Step {
    let Some(place) = MOVES.iter().position(|&k| k == key) else {
        return Step::Other { below: key < 0x69 };
    };
    if table {
        let to = entry(place);
        return if (to as i8) < 0 {
            Step::Blocked(to)
        } else {
            Step::Moved(to)
        };
    }
    match key {
        0x66 => {
            let to = from.wrapping_sub(1);
            Step::Moved(if (to as i8) < 0 {
                count.wrapping_sub(1)
            } else {
                to
            })
        }
        0x6B => {
            let to = from.wrapping_add(1);
            Step::Moved(if to < count { to } else { 0 })
        }
        _ => Step::NoTable,
    }
}

/// `menu::list_take_key`'s repeat count after a key: reset unless it is the last key
/// again and `frame - (moved + 6)` is negative (signed); then one more.
pub fn repeats(count: u32, again: bool, frame: u32, moved: u32) -> u32 {
    let reset = !again || (frame.wrapping_sub(moved.wrapping_add(6)) as i32) >= 0;
    if reset { 1 } else { count.wrapping_add(1) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marks() {
        assert_eq!(unmarked(0xC2), 2);
        assert!(has_choices(0xC2) && !has_choices(0x13));
        assert_eq!((rounds(0), rounds(3)), (0x100, 3));
        assert!(unusable(0x40) && unusable(0x81) && unusable(0x05));
        assert!(!unusable(0xC0) && !unusable(0x02));
    }

    #[test]
    fn steps() {
        let none = |_| unreachable!();
        assert_eq!(step(0x66, 0, 5, false, none), Step::Moved(4));
        assert_eq!(step(0x6B, 4, 5, false, none), Step::Moved(0));
        assert_eq!(step(0x6B, 2, 5, false, none), Step::Moved(3));
        assert_eq!(step(0x68, 2, 5, false, none), Step::NoTable);
        assert_eq!(step(0x10, 2, 5, false, none), Step::Other { below: true });
        assert_eq!(step(0x70, 2, 5, true, none), Step::Other { below: false });
        let table = [7u8, 0x80, 1, 2];
        assert_eq!(step(0x66, 2, 5, true, |k| table[k]), Step::Moved(7));
        assert_eq!(step(0x6B, 2, 5, true, |k| table[k]), Step::Blocked(0x80));
        assert_eq!(step(0x69, 2, 5, true, |k| table[k]), Step::Moved(2));
    }

    #[test]
    fn repeat() {
        assert_eq!(repeats(5, false, 0, 0), 1);
        assert_eq!(repeats(5, true, 10, 0), 1);
        assert_eq!(repeats(5, true, 5, 0), 6);
    }
}
