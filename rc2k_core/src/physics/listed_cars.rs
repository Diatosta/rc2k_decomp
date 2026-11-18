//! The cars listed for a device or sound slot (eight, by car; -1 none), and the cues sent to a
//! listed car's sound object.

use crate::Invalid;

/// No entry.
pub const NOT_LISTED: u32 = u32::MAX;

/// The current car's entry (a car past 7 is refused), and whether it is listed.
pub fn listed_entry(car: u32, entry: impl FnOnce(u32) -> u32) -> Result<(u32, bool), Invalid> {
    if car > 7 {
        return Err(Invalid("a current car past 7"));
    }
    let e = entry(car);
    Ok((e, e != NOT_LISTED))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CuePlace {
    /// Queued first.
    First = 0,
    /// Queued last (the pace notes).
    Last = 1,
}

/// A sound object's in-use flag value: it takes cues.
pub const READY: u32 = 1;

/// For `which` below `players` (the cars whose co-driver speaks: the first two, or a split
/// screen's three or four; else nothing): the car's sound object when it is `READY` (a car past
/// 7 is refused).
pub fn cue_object<O: Copy>(
    which: u32,
    players: u32,
    car_of: impl FnOnce(u32) -> u32,
    object_of: impl FnOnce(u32) -> O,
    ready_of: impl FnOnce(O) -> u32,
) -> Result<Option<O>, Invalid> {
    if which >= players {
        return Ok(None);
    }
    let car = car_of(which);
    if car > 7 {
        return Err(Invalid("a car past 7"));
    }
    let object = object_of(car);
    Ok((ready_of(object) == READY).then_some(object))
}

/// The stores of cue 0xE5 into the current car's sound object: (offset, value).
pub const CUE_E5: [(u32, u32); 3] = [(0x24C, 0xE5), (0x250, 0), (0x10, 1)];

/// The current car's sound object number when the car is listed; a car past 7 is refused.
pub fn cue_e5_object(car: u32, entry: impl FnOnce(u32) -> u32) -> Result<Option<u32>, Invalid> {
    let (_, listed) = listed_entry(car, entry)?;
    Ok(listed.then_some(car & 7))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entries() {
        assert_eq!(listed_entry(3, |c| c * 2), Ok((6, true)));
        assert_eq!(listed_entry(7, |_| NOT_LISTED), Ok((NOT_LISTED, false)));
        assert!(listed_entry(8, |_| unreachable!()).is_err());
    }

    #[test]
    fn cues() {
        assert_eq!(cue_object(2, 2, |_| unreachable!(), |_| 0, |_| 0), Ok(None));
        assert!(cue_object(1, 2, |_| 8, |_| 0, |_| 0).is_err());
        assert_eq!(
            cue_object(0, 2, |w| w + 3, |c| c * 0x100, |_| READY),
            Ok(Some(0x300))
        );
        assert_eq!(cue_object(1, 2, |w| w + 3, |c| c * 0x100, |_| 0), Ok(None));
        // A split screen's third car speaks too.
        assert_eq!(cue_object(2, 3, |w| w, |c| c, |_| READY), Ok(Some(2)));
        assert_eq!(cue_object(3, 3, |_| unreachable!(), |_| 0, |_| 0), Ok(None));
        assert_eq!(cue_e5_object(2, |_| NOT_LISTED), Ok(None));
        assert_eq!(cue_e5_object(2, |_| 0), Ok(Some(2)));
        assert!(cue_e5_object(9, |_| 0).is_err());
    }
}
