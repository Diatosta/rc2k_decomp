//! The class pages' passes: which classes are walked and which have a page.

use crate::Invalid;
use std::ops::Range;

/// The most classes the passes take.
pub const MOST_CLASSES: u32 = 0x100;

/// `MenuState.menus.class_mode` when the class pages start at class 1.
pub const FROM_CLASS_1: u32 = 2;

/// The A8 championship's event mode.
pub const A8: u32 = 0x13;

/// The stage results' first class: 1 with `MenuState.menus.class_mode` 2, else 0.
pub fn first_stage_class(mode: u32) -> u32 {
    u32::from(mode == FROM_CLASS_1)
}

/// The rally results' first class.
pub const FIRST_RALLY_CLASS: u32 = 0;

/// The championship's first class: 0 in the A8 championship when `mode`
/// (`MenuState.menus.class_mode`) is not 2, else 1; `event_mode` is asked only when it is not 2.
pub fn first_championship_class(mode: u32, event_mode: impl FnOnce() -> u32) -> u32 {
    if mode != FROM_CLASS_1 && event_mode() == A8 {
        0
    } else {
        1
    }
}

/// The classes walked, `first` up to `count`. Refused: a count the loop would run past
/// (at or below the first, or past `MOST_CLASSES`).
pub fn classes(first: u32, count: u32) -> Result<Range<u32>, Invalid> {
    if count <= first || count > MOST_CLASSES {
        return Err(Invalid("a class count the loop would run past"));
    }
    Ok(first..count)
}

/// A class has a page when a player (an entry's +4 1 .. 3: player 4 is not looked at) is
/// in it; of the `entries`, `class_of` gives each one's class and `who` its +4 (asked only for
/// those in the class).
pub fn has_player(
    entries: usize,
    class_of: impl Fn(usize) -> u32,
    who: impl Fn(usize) -> u32,
    class: u32,
) -> bool {
    (0..entries).any(|n| class_of(n) == class && (1..=3).contains(&who(n)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classes_and_pages() {
        assert_eq!(first_stage_class(2), 1);
        assert_eq!(first_stage_class(1), 0);
        assert_eq!(first_championship_class(2, || unreachable!()), 1);
        assert_eq!(first_championship_class(1, || 0x13), 0);
        assert_eq!(first_championship_class(1, || 2), 1);
        assert_eq!(classes(1, 3), Ok(1..3));
        assert!(classes(1, 1).is_err() && classes(0, 0x101).is_err());
        let e = [(0, 0), (1, 4), (2, 3)];
        let has = |c| has_player(3, |n| e[n].0, |n| e[n].1, c);
        assert!(!has(0) && !has(1) && has(2));
    }
}
