//! A sound object's queue of cues (converted 2026-10-21 from `physics::queue_sound_cue`
//! 0x40EE5C): up to 16 entries of 0x20 bytes (the cue, its sounds' number, up to six
//! sounds); a cue put first stops the playing sound and moves the others up one.

use crate::Invalid;

/// The queue's room, and an entry's size in dwords.
pub const ROOM: u32 = 16;
pub const ENTRY_DWORDS: u32 = 8;

/// The highest cue: `off_512508` holds 113 sound lists, cues 0 ..= 0x70, ended by -1. The
/// original does not check; a cue past the table would read past it.
pub const LAST_CUE: u32 = 0x70;

/// The most sounds a cue lists, and the end of a list.
pub const MOST_SOUNDS: u32 = 6;
pub const LIST_END: u32 = u32::MAX;

/// Where a cue goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CuePlacement {
    /// The queue is full: nothing.
    Full,
    /// At entry `n` (0 into an empty queue or last).
    At(u32),
    /// First, ahead of the `count` queued (the playing sound stopped, the entries moved up).
    First { count: u32 },
}

/// A cue past `LAST_CUE` is refused (before anything is read).
pub fn check_cue(cue: u32) -> Result<(), Invalid> {
    if cue > LAST_CUE {
        return Err(Invalid("a sound cue past 0x70"));
    }
    Ok(())
}

/// The place of a cue (`first`: eax 0) in a queue of `count`.
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

/// A cue's sounds: the list read in order (`list(k)`) up to a -1 or `MOST_SOUNDS`, each
/// not below 0 offset by the object's base; (the dwords written, the -1 included; their
/// number less the -1, the entry's +4).
pub fn sounds(list: impl Fn(u32) -> u32, base: u32) -> (Vec<u32>, u32) {
    let mut out = Vec::new();
    let mut n = 0;
    loop {
        let mut s = list(n);
        if (s as i32) >= 0 {
            s = s.wrapping_add(base);
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
