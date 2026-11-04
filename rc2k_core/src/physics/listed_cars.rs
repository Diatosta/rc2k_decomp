//! The cars listed for a device or sound slot (converted 2026-10-21 from the three
//! twins `physics::set_flag_51659c` 0x45D151, `set_flag_5165a0` 0x45D16D and
//! `set_flag_5165a4` 0x45D189): a flag set when the current car has an entry in
//! `dword_641680` (eight, by car; -1 none). And the cues sent to a listed car's sound
//! object (converted 2026-10-21 from `physics::cue_car_sound` 0x40ECE7,
//! `cue_car_sound_last` 0x40ECBB, `cue_listed_car_sound` 0x45D1BC,
//! `cue_listed_car_sound_last` 0x45D1D3 and `sound_cue_e5` 0x45D028): the sound objects
//! (`g_raceSoundObjects`, eight, by car) and which cars' cues are heard.

use crate::Invalid;

/// No entry.
pub const NOT_LISTED: u32 = u32::MAX;

/// The current car's entry, `entry(car)` asked once the car is known to be one of the
/// eight (past 7 is refused: the table's size is not worked out); and whether the flag
/// is set by it (the entry not -1).
pub fn listed_entry(car: u32, entry: impl FnOnce(u32) -> u32) -> Result<(u32, bool), Invalid> {
    if car > 7 {
        return Err(Invalid("a current car past 7"));
    }
    let e = entry(car);
    Ok((e, e != NOT_LISTED))
}

/// Where a cue goes in the object's queue (the original's eax for
/// `physics::queue_sound_cue`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CuePlace {
    /// `physics::cue_car_sound`: first (0).
    First = 0,
    /// `physics::cue_car_sound_last` (the pace notes): last (1).
    Last = 1,
}

/// A sound object's +8: 1 when it takes cues (a reading: ready).
pub const READY: u32 = 1;
/// The object's queue (+0x298), what `queue_sound_cue` is given in edi.
pub const QUEUE_OFFSET: u32 = 0x298;

/// `physics::cue_car_sound` and `_last`: for `which` 0 or 1 (unsigned; else nothing), the
/// car `car_of(which)` (past 7 refused), its sound object `object_of(car)` (a handle of
/// the caller's, never null), its +8 `ready_of(object)`: the object when that is `READY`.
pub fn cue_object<O: Copy>(
    which: u32,
    car_of: impl FnOnce(u32) -> u32,
    object_of: impl FnOnce(u32) -> O,
    ready_of: impl FnOnce(O) -> u32,
) -> Result<Option<O>, Invalid> {
    if which > 1 {
        return Ok(None);
    }
    let car = car_of(which);
    if car > 7 {
        return Err(Invalid("a car past 7"));
    }
    let object = object_of(car);
    Ok((ready_of(object) == READY).then_some(object))
}

/// `physics::sound_cue_e5`'s stores into the current car's sound object (a reading: the
/// sound 0xE5 to play): (offset, value) in the original's order.
pub const CUE_E5: [(u32, u32); 3] = [(0x24C, 0xE5), (0x250, 0), (0x10, 1)];

/// `physics::sound_cue_e5`: the current car's sound object number (`car & 7`) when the
/// car is listed; none when not; a car past 7 refused first.
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
        assert_eq!(cue_object(2, |_| unreachable!(), |_| 0, |_| 0), Ok(None));
        assert!(cue_object(1, |_| 8, |_| 0, |_| 0).is_err());
        assert_eq!(
            cue_object(0, |w| w + 3, |c| c * 0x100, |_| READY),
            Ok(Some(0x300))
        );
        assert_eq!(cue_object(1, |w| w + 3, |c| c * 0x100, |_| 0), Ok(None));
        assert_eq!(cue_e5_object(2, |_| NOT_LISTED), Ok(None));
        assert_eq!(cue_e5_object(2, |_| 0), Ok(Some(2)));
        assert!(cue_e5_object(9, |_| 0).is_err());
    }
}
