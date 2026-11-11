//! The pace notes called as the car goes (from
//! `physics::pace_notes_step` 0x4417EC (its whole-number rules; the look-ahead's
//! `fistp` stays x87) and `physics::queue_pace_note` 0x4418B3): the checkpoints ahead of
//! the car up to which notes are called, and each checkpoint's group of codes, cued and
//! queued up to a NUL (a code of 0x54 or more is bad data, logged).

use crate::Invalid;

/// The start timer from which notes are called (unsigned).
pub const CALL_FROM: u32 = 1000;

/// Whether notes are called this step: the seven gates 0, the grades' mode 1, the start
/// timer at least `CALL_FROM`.
pub fn calling(gates: impl IntoIterator<Item = u32>, mode: u32, timer: u32) -> bool {
    gates.into_iter().all(|g| g == 0) && mode == 1 && timer >= CALL_FROM
}

/// The checkpoint up to which notes are called: the car's plus the look-ahead when that
/// is not negative, 0 when at or past the count (by the sign).
pub fn call_target(checkpoint: u32, ahead: i32, count: u32) -> u32 {
    let mut t = checkpoint;
    if ahead >= 0 {
        t = t.wrapping_add(ahead as u32);
    }
    if (t.wrapping_sub(count) as i32) >= 0 {
        t = 0;
    }
    t
}

/// The most checkpoints called in one step; past it the last called jumps to the target.
pub const MOST_CALLED: u32 = 0x10;

/// The checkpoints whose groups are called: (the last called as the calls begin, the
/// difference when it was worked out); None when the target is not
/// past the last called (unsigned). The calls run from the first after it up to the
/// target, at least one.
pub fn calls(target: u32, last: u32) -> Option<(u32, u32)> {
    if target <= last {
        return None;
    }
    let gap = target - last;
    Some((if gap > MOST_CALLED { target } else { last }, gap))
}

/// The most groups (`dword_619E18`'s room).
pub const MOST_GROUPS: u32 = 0x400;

/// The first group called at `checkpoint` (`checkpoint_of(n)` read in order); None when
/// there are no groups or none is at it (`Err` past `MOST_GROUPS`).
pub fn find_group(
    groups: u32,
    checkpoint: u32,
    checkpoint_of: impl Fn(u32) -> u32,
) -> Result<Option<u32>, Invalid> {
    if groups == 0 {
        return Ok(None);
    }
    if groups > MOST_GROUPS {
        return Err(Invalid("more than 0x400 pace note groups"));
    }
    Ok((0..groups).find(|&n| checkpoint_of(n) == checkpoint))
}

/// The texts' room (`byte_61BE18`).
pub const TEXT_LEN: u32 = 0x4000;
/// The first bad code.
pub const BAD_CODE: u32 = 0x54;

/// A code of a group's text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Code {
    /// The NUL: the group ends.
    End,
    /// 0x54 or more: logged, the group ends.
    Bad,
    /// A note: cued and queued.
    Call,
}

/// A text code's meaning.
pub fn code(byte: u8) -> Code {
    match u32::from(byte) {
        0 => Code::End,
        b if b >= BAD_CODE => Code::Bad,
        _ => Code::Call,
    }
}

/// A text place checked before its code is read.
pub fn text_place(place: u32) -> Result<u32, Invalid> {
    if place >= TEXT_LEN {
        return Err(Invalid("a pace note text place past byte_61BE18's 0x4000"));
    }
    Ok(place)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps() {
        assert!(calling([0; 7], 1, 1000));
        assert!(!calling([0, 0, 1], 1, 1000));
        assert!(!calling([0; 7], 1, 999));
        assert_eq!(call_target(10, 5, 100), 15);
        assert_eq!(call_target(10, -5, 100), 10);
        assert_eq!(call_target(98, 5, 100), 0);
        assert_eq!(calls(15, 15), None);
        assert_eq!(calls(15, 10), Some((10, 5)));
        assert_eq!(calls(40, 10), Some((40, 30)));
    }

    #[test]
    fn groups() {
        assert_eq!(find_group(0, 3, |_| unreachable!()), Ok(None));
        assert_eq!(find_group(4, 3, |n| n + 1), Ok(Some(2)));
        assert_eq!(find_group(4, 9, |n| n), Ok(None));
        assert!(find_group(0x401, 3, |_| unreachable!()).is_err());
        assert_eq!(code(0), Code::End);
        assert_eq!(code(0x53), Code::Call);
        assert_eq!(code(0x54), Code::Bad);
        assert!(text_place(0x4000).is_err());
    }
}
