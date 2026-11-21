//! The pace notes called as the car goes: the checkpoint up to which notes are called, and each
//! checkpoint's group of codes, cued and queued up to a NUL (a code of 0x54 or more is bad
//! data, logged).

use crate::Invalid;

/// The start timer from which notes are called (unsigned).
pub(crate) const CALL_FROM: u32 = 1000;

/// Whether notes are called this step: all gates 0, mode 1, the start timer at `CALL_FROM`.
pub fn calling(gates: impl IntoIterator<Item = u32>, mode: u32, timer: u32) -> bool {
    gates.into_iter().all(|g| g == 0) && mode == 1 && timer >= CALL_FROM
}

/// The checkpoint up to which notes are called: the car's plus the look-ahead when that is
/// not negative, 0 at or past the count.
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
pub(crate) const MOST_CALLED: u32 = 0x10;

/// The checkpoints whose groups are called: (the last called as the calls begin, the
/// difference); `None` when the target is not past the last called (unsigned). The calls run
/// from the first checkpoint after it up to the target, at least one.
pub fn calls(target: u32, last: u32) -> Option<(u32, u32)> {
    if target <= last {
        return None;
    }
    let gap = target - last;
    Some((if gap > MOST_CALLED { target } else { last }, gap))
}

/// The most groups.
pub(crate) const MOST_GROUPS: u32 = 0x400;

/// The first group called at `checkpoint`; `None` when there are none or none is at it
/// (an error past `MOST_GROUPS`).
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

/// The texts' room.
pub const TEXT_LEN: u32 = 0x4000;
/// The first bad code.
pub(crate) const BAD_CODE: u32 = 0x54;

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
        return Err(Invalid(
            "a pace note text place past the text buffer's 0x4000",
        ));
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
