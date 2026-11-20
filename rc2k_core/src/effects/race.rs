//! What the marks take from the race: each car's record set up for the marks when a stage loads,
//! and the stage's record among the 0x34.

use crate::Invalid;

/// The most cars of a race.
pub const MOST_CARS: u32 = 8;
/// The last kind of car with a row (`LightsAndGear.light_records.records[0].groups[5]` ..: rows of
/// 0x134 bytes).
pub const LAST_KIND: i32 = 0x16;
pub const ROW_LEN: usize = 0x134;
/// The stages' records.
pub(crate) const STAGE_RECORDS: u32 = 0x34;

/// The groups of a kind's row (counted from the row's first group) the record's `slot_kinds` (its
/// first seven) are set from.
pub const SLOT_KIND_GROUPS: [usize; 7] = [0, 8, 9, 6, 7, 5, 10];
/// The seventeen numbers of the record's `kinds`.
pub(crate) const FIXED: [u32; 17] = [
    0xA, 0xB, 0x18, 0x19, 0xC, 0xD, 0x16, 0x17, 0x1A, 0x1B, 0xE, 0xF, 0x10, 0x11, 0x13, 0x14, 0x12,
];

/// A car's record fields set when a stage loads.
#[derive(Debug, PartialEq, Eq)]
pub struct CarFields {
    /// The record's `object`.
    pub object: u32,
    /// The record's `second_object`.
    pub second_object: u32,
    /// The record's first seven `slot_kinds`, from the kind's row.
    pub slot_kinds: [u32; 7],
    /// The record's `kinds`.
    pub kinds: [u32; 17],
}

/// `init_mark_step`'s check, before anything: refuses no cars or more than 8 (the loop would
/// run on) and a car without its block or record (`has(n)`).
pub fn check_cars(count: u32, has: impl Fn(u32) -> bool) -> Result<(), Invalid> {
    if count == 0 || count > MOST_CARS {
        return Err(Invalid("an init mark of 0 or more than 8 cars"));
    }
    if (0..count).any(|n| !has(n)) {
        return Err(Invalid("an init mark car without its block or record"));
    }
    Ok(())
}

/// A kind's row (its block's kind held to 0 .. 0x16): its offset in the rows.
pub fn kind_row(kind: i32) -> usize {
    kind.clamp(0, LAST_KIND) as usize * ROW_LEN
}

/// Car n's record's fields, the places first (0x50 + 0x14 a car, and 0xA more), the seven
/// from its kind's row (`row(group)`, asked in the order of `SLOT_KIND_GROUPS`), the
/// seventeen fixed numbers.
pub fn car_fields(n: u32, row: impl Fn(usize) -> u32) -> CarFields {
    let place = n * 0x14 + 0x50;
    CarFields {
        object: place,
        second_object: place + 0xA,
        slot_kinds: SLOT_KIND_GROUPS.map(row),
        kinds: FIXED,
    }
}

/// `stage_record`: the record of a stage's number; refuses one past the table (unsigned).
pub fn stage_record(number: u32) -> Result<usize, Invalid> {
    if number >= STAGE_RECORDS {
        return Err(Invalid("a stage number past the 0x34"));
    }
    Ok(number as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cars() {
        assert!(check_cars(0, |_| true).is_err());
        assert!(check_cars(9, |_| true).is_err());
        assert!(check_cars(3, |n| n != 2).is_err());
        assert!(check_cars(8, |_| true).is_ok());
        assert_eq!(kind_row(-1), 0);
        assert_eq!(kind_row(0x17), 0x16 * 0x134);
        let f = car_fields(2, |g| g as u32 * 4 + 1);
        assert_eq!((f.object, f.second_object), (0x78, 0x82));
        assert_eq!(f.slot_kinds, [1, 0x21, 0x25, 0x19, 0x1D, 0x15, 0x29]);
        assert_eq!(f.kinds[16], 0x12);
        assert_eq!(stage_record(0x33), Ok(0x33));
        assert!(stage_record(0x34).is_err());
    }
}
