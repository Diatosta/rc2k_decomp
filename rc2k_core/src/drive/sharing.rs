//! The cars' state shared out for the drawing and the sounds (from
//! `drive::share_car_state` 0x45699C and `drive::share_all_cars` 0x456484; the levels'
//! x87 sums stay with the caller): each car's block (0xFE0 bytes) into its record (0x440,
//! `g_carRecords[n]`) and back, field by field as the original moves them.

use crate::Invalid;

/// The light records (`g_lightRecords`, 0x134 bytes each).
pub const LIGHT_RECORDS: u32 = 0x22;

/// A car file number (`race::car_file_number` of block +0x24) with a light record.
pub fn check_light_record(n: u32) -> Result<(), Invalid> {
    if n >= LIGHT_RECORDS {
        return Err(Invalid(
            "a car file number above 0x21 (past g_lightRecords)",
        ));
    }
    Ok(())
}

/// The notice count (block +0xFAC): when not 0, the record's +0x30 set and the count
/// less one.
pub fn count_down(count: u32) -> Option<u32> {
    (count != 0).then(|| count - 1)
}

/// A level (record +0x4C0, +0x4C4) from the x87's whole number: held to 0 ..= 7 (by the
/// sign of the difference).
pub fn level(v: i32) -> u32 {
    let v = if v < 0 { 0 } else { v };
    let v = if v.wrapping_sub(7) < 0 { v } else { 7 };
    v as u32
}

/// Whether wheel k's record word is cleared: its switch (block +0x600 + 4k) 0 and bit
/// 5 + k of block +0x298 set.
pub fn wheel_cleared(switch: u32, bits: u32, k: u32) -> bool {
    switch == 0 && bits >> (5 + k) & 1 != 0
}

/// A hole in the six dwords copied to block +0x700.
pub const HOLE: u32 = 0x8000_0000;

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

/// `drive::share_all_cars`: `ViewerAndKinds.marks_settings.sound_objects_shown`, the cars shown: the lead car's file
/// not 0x21 and the first record's +0x258 (`laps()`, read only then) 10
/// or more. (the flag, that word when read).
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
