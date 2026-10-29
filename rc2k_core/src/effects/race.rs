//! What the marks take from the race (converted 2026-10-21 from `effects::init_mark_step`
//! 0x456EA7 and `effects::stage_record` 0x4D2DE0): each car's record set up for the marks
//! when a stage loads, and the stage's record among the 0x34.

use crate::Invalid;

/// The most cars of a race.
pub const MOST_CARS: u32 = 8;
/// The last kind of car with a row (`dword_5C90D8` ..: rows of 0x134 bytes).
pub const LAST_KIND: i32 = 0x16;
pub const ROW_LEN: usize = 0x134;
/// The stages' records (`off_606320`).
pub const STAGE_RECORDS: u32 = 0x34;

/// The record's fields set from the kind's row: (the field, the row's offset).
pub const COPIES: [(u32, usize); 7] = [
    (0x30C, 0),
    (0x310, 0x20),
    (0x314, 0x24),
    (0x318, 0x18),
    (0x31C, 0x1C),
    (0x320, 0x14),
    (0x324, 0x28),
];
/// The seventeen numbers at +0x36C ...
pub const FIXED: [u32; 17] = [
    0xA, 0xB, 0x18, 0x19, 0xC, 0xD, 0x16, 0x17, 0x1A, 0x1B, 0xE, 0xF, 0x10, 0x11, 0x13, 0x14, 0x12,
];

/// `init_mark_step`'s check, before anything: refuses no cars or more than 8 (the original's
/// loop would run on) and a car without its block or record (`has(n)`).
pub fn check_cars(count: u32, has: impl Fn(u32) -> bool) -> Result<(), Invalid> {
    if count == 0 || count > MOST_CARS {
        return Err(Invalid("an init mark of 0 or more than 8 cars"));
    }
    if (0..count).any(|n| !has(n)) {
        return Err(Invalid("an init mark car without its block or record"));
    }
    Ok(())
}

/// A kind's row (its block's +0x24 held to 0 .. 0x16): its offset in the rows.
pub fn kind_row(kind: i32) -> usize {
    kind.clamp(0, LAST_KIND) as usize * ROW_LEN
}

/// Car n's record's fields, in the order written: its places +0x23C (0x50 + 0x14 a car)
/// and +0x240 (0xA more), the seven from its kind's row (`row(offset)`), the seventeen
/// fixed numbers.
pub fn car_fields(n: u32, row: impl Fn(usize) -> u32) -> Vec<(u32, u32)> {
    let place = n * 0x14 + 0x50;
    let mut out = vec![(0x23C, place), (0x240, place + 0xA)];
    out.extend(COPIES.map(|(field, offset)| (field, row(offset))));
    out.extend(
        FIXED
            .iter()
            .enumerate()
            .map(|(k, &v)| (0x36C + 4 * k as u32, v)),
    );
    out
}

/// `stage_record`: the record of a stage's number; refuses one past the table (unsigned).
pub fn stage_record(number: u32) -> Result<usize, Invalid> {
    if number >= STAGE_RECORDS {
        return Err(Invalid("a stage number past off_606320's 0x34"));
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
        let f = car_fields(2, |o| o as u32 + 1);
        assert_eq!(f.len(), 2 + 7 + 17);
        assert_eq!(&f[..3], &[(0x23C, 0x78), (0x240, 0x82), (0x30C, 1)]);
        assert_eq!(f[25], (0x36C + 64, 0x12));
        assert_eq!(stage_record(0x33), Ok(0x33));
        assert!(stage_record(0x34).is_err());
    }
}
