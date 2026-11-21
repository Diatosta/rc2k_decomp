//! The cars' state shared out for the drawing and the sounds: each car's block (0xFE0 bytes) into
//! its car record (0x440 bytes) and back, field by field. The levels' float sums stay with the
//! caller.

use crate::Invalid;

/// The light records' count (0x134 bytes each).
pub const LIGHT_RECORDS: u32 = 0x22;

/// Refused: a car file number (`race::car_file_number` of the block's car) without a light record.
pub fn check_light_record(n: u32) -> Result<(), Invalid> {
    if n >= LIGHT_RECORDS {
        return Err(Invalid(
            "a car file number above 0x21 (past g_lightRecords)",
        ));
    }
    Ok(())
}

/// The block's notice count: when not 0, the record's notice flag is set and the count
/// is one less.
pub fn count_down(count: u32) -> Option<u32> {
    (count != 0).then(|| count - 1)
}

/// A level of the record from the caller's rounded float: held to 0 ..= 7.
pub fn level(v: i32) -> u32 {
    let v = if v < 0 { 0 } else { v };
    let v = if v.wrapping_sub(7) < 0 { v } else { 7 };
    v as u32
}

/// Whether wheel k's record word is cleared: the wheel's switch in the block 0 and bit
/// 5 + k of the block's wheel bits set.
pub fn wheel_cleared(switch: u32, bits: u32, k: u32) -> bool {
    switch == 0 && bits >> (5 + k) & 1 != 0
}

/// A hole in the six dwords copied into the block.
pub(crate) const HOLE: u32 = 0x8000_0000;

/// The holes filled in place, in order: each from the next entry (round) that is not a
/// hole, at most six looked at (all holes: it stays).
pub fn fill_holes(values: &mut [u32; 6]) {
    for c in 0..6 {
        if values[c] != HOLE {
            continue;
        }
        let mut c2 = c;
        for _ in 0..6 {
            if values[c2] != HOLE {
                break;
            }
            c2 = (c2 + 1) % 6;
        }
        values[c] = values[c2];
    }
}

/// Whether the cars are shown (`ViewerAndKinds.marks_settings.sound_objects_shown`): the lead
/// car's file not 0x21 and the first record's lap word (`laps()`, read only then) 10 or more.
/// Gives (the flag, that word when read).
pub fn cars_shown(lead_file: u32, laps: impl FnOnce() -> u32) -> (bool, Option<u32>) {
    if lead_file == 0x21 {
        return (false, None);
    }
    let w = laps();
    (w >= 10, Some(w))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules() {
        assert!(check_light_record(0x21).is_ok() && check_light_record(0x22).is_err());
        assert_eq!(count_down(0), None);
        assert_eq!(count_down(2), Some(1));
        assert_eq!(level(-3), 0);
        assert_eq!(level(5), 5);
        assert_eq!(level(9), 7);
        assert!(wheel_cleared(0, 1 << 6, 1));
        assert!(!wheel_cleared(1, 1 << 6, 1));
        assert!(!wheel_cleared(0, 1 << 6, 0));
        assert_eq!(cars_shown(0x21, || panic!()), (false, None));
        assert_eq!(cars_shown(3, || 10), (true, Some(10)));
        assert_eq!(cars_shown(3, || 9), (false, Some(9)));
    }

    #[test]
    fn holes() {
        let mut v = [HOLE, 1, HOLE, HOLE, 4, HOLE];
        fill_holes(&mut v);
        assert_eq!(v, [1, 1, 4, 4, 4, 1]);
        let mut v = [HOLE; 6];
        fill_holes(&mut v);
        assert_eq!(v, [HOLE; 6]);
    }
}
