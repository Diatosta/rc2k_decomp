//! A sound object's queue of cues: up to 16 entries of 0x20 bytes; a cue put first stops the
//! playing sound and moves the others up one.

use crate::Invalid;
use crate::audio::voices::codriver_copy;

/// The queue's room.
pub const ROOM: u32 = 16;
/// An entry's size in dwords.
pub const ENTRY_DWORDS: u32 = 8;

/// The highest cue: the table holds cues 0 ..= 0x70.
pub(crate) const LAST_CUE: u32 = 0x70;

/// The most sounds a cue lists.
pub(crate) const MOST_SOUNDS: u32 = 6;
/// The end of a sound list.
pub(crate) const LIST_END: u32 = u32::MAX;

/// Where a cue goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CuePlacement {
    /// The queue is full: nothing.
    Full,
    /// At entry `n` (0 into an empty queue, or last).
    At(u32),
    /// First, ahead of the `count` queued (the playing sound stopped).
    First { count: u32 },
}

/// A cue past `LAST_CUE` is refused.
pub fn check_cue(cue: u32) -> Result<(), Invalid> {
    if cue > LAST_CUE {
        return Err(Invalid("a sound cue past 0x70"));
    }
    Ok(())
}

/// The place of a cue (`first` set) in a queue of `count`.
pub fn place(count: u32, first: bool) -> CuePlacement {
    if count == ROOM {
        CuePlacement::Full
    } else if count == 0 {
        CuePlacement::At(0)
    } else if first {
        CuePlacement::First { count }
    } else {
        CuePlacement::At(count)
    }
}

/// A cue's sound as the object whose sounds are offset by `base` says it: the list's sound
/// moved on by the base (0 and 1: the first two players' voices, a pair of records), the split
/// screen's third and fourth players' (2 and 3) the same pair's copies.
pub fn cue_sound(sound: u32, base: u32) -> u32 {
    if matches!(base, 2 | 3) {
        let pair = sound.wrapping_add(base & 1);
        if let Some(copy) = codriver_copy(pair) {
            return copy;
        }
    }
    sound.wrapping_add(base)
}

/// A cue's sounds: the list read up to a -1 or `MOST_SOUNDS`, each not below 0 given to the
/// object whose base is `base` (`cue_sound`); returns (the dwords written, their number less the
/// -1).
pub fn sounds(list: impl Fn(u32) -> u32, base: u32) -> (Vec<u32>, u32) {
    let mut out = Vec::new();
    let mut n = 0;
    loop {
        let mut s = list(n);
        if (s as i32) >= 0 {
            s = cue_sound(s, base);
        }
        out.push(s);
        if s == LIST_END {
            break;
        }
        n += 1;
        if n == MOST_SOUNDS {
            break;
        }
    }
    (out, n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn third_and_fourth_voices() {
        use crate::audio::voices::{CODRIVER_COPIES as copies, CODRIVER_FIRST};
        assert_eq!(cue_sound(CODRIVER_FIRST + 4, 0), CODRIVER_FIRST + 4);
        assert_eq!(cue_sound(CODRIVER_FIRST + 4, 1), CODRIVER_FIRST + 5);
        assert_eq!(cue_sound(CODRIVER_FIRST + 4, 2), copies + 4);
        assert_eq!(cue_sound(CODRIVER_FIRST + 4, 3), copies + 5);
        // A sound that is no voice keeps the plain offset.
        assert_eq!(cue_sound(10, 2), 12);
    }

    #[test]
    fn places() {
        assert_eq!(place(16, true), CuePlacement::Full);
        assert_eq!(place(0, true), CuePlacement::At(0));
        assert_eq!(place(4, true), CuePlacement::First { count: 4 });
        assert_eq!(place(4, false), CuePlacement::At(4));
        assert!(check_cue(0x70).is_ok());
        assert!(check_cue(0x71).is_err());
    }

    #[test]
    fn lists() {
        let l = [1u32, 2, u32::MAX];
        assert_eq!(
            sounds(|k| l[k as usize], 0x100),
            (vec![0x101, 0x102, u32::MAX], 2)
        );
        assert_eq!(sounds(|k| k, 0), (vec![0, 1, 2, 3, 4, 5], 6));
        // A base that makes a sound -1 ends the list there.
        assert_eq!(sounds(|_| 1, u32::MAX - 1), (vec![u32::MAX], 0));
    }
}
